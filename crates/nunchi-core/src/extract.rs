//! tree-sitter 심볼 추출 — 빠른 경로 (PLAN.md 3.9절 2단 속도 인덱싱)
//!
//! 파일 저장 시마다 도는 경로이므로 밀리초 단위여야 한다. 크로스파일 참조 해소는
//! 이름 기반 휴리스틱이며, 정밀 해소는 SCIP 경로(Phase 1b)가 맡는다.
//! 그래서 여기서 만든 엣지는 모두 `Provenance::Fast`다.

use crate::model::Span;
use anyhow::{Context, Result};
use std::path::Path;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Node, Parser, Query, QueryCursor};

/// 파일 하나에서 뽑아낸 사실. 브랜치가 아니라 **내용의 함수**이므로
/// 콘텐츠 주소 캐시(PLAN.md 3.7절)의 캐시 대상이 된다.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileFacts {
    pub symbols: Vec<SymbolFact>,
    pub imports: Vec<String>,
    pub calls: Vec<CallFact>,
    /// 파서가 오류 노드를 만들었는지. `nunchi doctor` 커버리지에 쓴다.
    pub had_parse_error: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SymbolFact {
    pub name: String,
    /// `function`, `class`, `method` 등 — 쿼리의 `@def.<kind>` 캡처에서 온다
    pub kind: String,
    pub span: Span,
    pub signature: Option<String>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CallFact {
    pub callee: String,
    pub line: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SupportedLang {
    Java,
    TypeScript,
    Rust,
}

impl SupportedLang {
    pub fn from_name(lang: &str) -> Option<Self> {
        match lang {
            "java" => Some(Self::Java),
            "typescript" | "javascript" => Some(Self::TypeScript),
            "rust" => Some(Self::Rust),
            _ => None,
        }
    }

    pub fn language_for(self, path: &Path) -> Language {
        self.language(path)
    }

    fn language(self, path: &Path) -> Language {
        match self {
            Self::Java => tree_sitter_java::LANGUAGE.into(),
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::TypeScript => {
                let is_tsx = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case("tsx") || e.eq_ignore_ascii_case("jsx"));
                if is_tsx {
                    tree_sitter_typescript::LANGUAGE_TSX.into()
                } else {
                    tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
                }
            }
        }
    }

    fn query_source(self) -> &'static str {
        match self {
            Self::Java => include_str!("../queries/java.scm"),
            Self::TypeScript => include_str!("../queries/typescript.scm"),
            Self::Rust => include_str!("../queries/rust.scm"),
        }
    }
}

/// 파일 하나를 추출한다. 파서가 실패해도 `Err`를 내지 않고 부분 결과를 돌려준다 —
/// 한 파일 때문에 전체 인덱싱이 멈추면 안 된다.
pub fn extract(lang: SupportedLang, path: &Path, source: &str) -> Result<FileFacts> {
    let language = lang.language(path);
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .context("tree-sitter 언어 설정 실패")?;
    let Some(tree) = parser.parse(source, None) else {
        return Ok(FileFacts { had_parse_error: true, ..Default::default() });
    };

    let query = Query::new(&language, lang.query_source())
        .with_context(|| format!("{lang:?} 쿼리 컴파일 실패"))?;

    let mut facts = FileFacts {
        had_parse_error: tree.root_node().has_error(),
        ..Default::default()
    };

    let bytes = source.as_bytes();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), bytes);

    while let Some(m) = matches.next() {
        let mut def_node: Option<(Node, &str)> = None;
        let mut name: Option<String> = None;

        for cap in m.captures {
            let cap_name = &query.capture_names()[cap.index as usize];
            let text = cap.node.utf8_text(bytes).unwrap_or_default();

            if let Some(kind) = cap_name.strip_prefix("def.") {
                def_node = Some((cap.node, kind));
            } else {
                match *cap_name {
                    "name" => name = Some(text.to_string()),
                    "callee" => facts.calls.push(CallFact {
                        callee: text.to_string(),
                        line: cap.node.start_position().row as u32 + 1,
                    }),
                    "import.path" => {
                        facts.imports.push(text.trim_matches(['"', '\'']).to_string())
                    }
                    _ => {}
                }
            }
        }

        if let (Some((node, kind)), Some(name)) = (def_node, name) {
            facts.symbols.push(SymbolFact {
                name,
                kind: kind.to_string(),
                span: Span {
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                },
                signature: first_line(node, bytes),
                doc: preceding_doc(node, bytes),
            });
        }
    }

    Ok(facts)
}

/// 시그니처 대용 — 정의의 첫 줄. 팩 렌더링 L0/L1 티어에서 쓴다(PLAN.md 3절).
fn first_line(node: Node, bytes: &[u8]) -> Option<String> {
    let text = node.utf8_text(bytes).ok()?;
    let line = text.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }
    const MAX: usize = 200;
    Some(if line.chars().count() > MAX {
        let truncated: String = line.chars().take(MAX).collect();
        format!("{truncated}…")
    } else {
        line.to_string()
    })
}

/// 정의 바로 앞의 주석을 문서로 간주한다.
fn preceding_doc(node: Node, bytes: &[u8]) -> Option<String> {
    let prev = node.prev_sibling()?;
    if !prev.kind().contains("comment") {
        return None;
    }
    let text = prev.utf8_text(bytes).ok()?.trim();
    const MAX: usize = 500;
    Some(if text.chars().count() > MAX {
        text.chars().take(MAX).collect()
    } else {
        text.to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 쿼리에 잘못된 노드 타입이 있으면 런타임에야 터진다. 여기서 먼저 잡는다.
    #[test]
    fn all_queries_compile() {
        for lang in [SupportedLang::Java, SupportedLang::TypeScript, SupportedLang::Rust] {
            let language = lang.language(Path::new("x.rs"));
            Query::new(&language, lang.query_source())
                .unwrap_or_else(|e| panic!("{lang:?} 쿼리 컴파일 실패: {e}"));
        }
    }

    #[test]
    fn extracts_rust_symbols() -> Result<()> {
        let src = r#"
/// 주문을 조회한다.
pub fn find_order(id: u32) -> Option<Order> {
    lookup(id)
}

pub struct Order { pub id: u32 }
"#;
        let f = extract(SupportedLang::Rust, Path::new("a.rs"), src)?;
        let names: Vec<_> = f.symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"find_order"), "got {names:?}");
        assert!(names.contains(&"Order"), "got {names:?}");

        let func = f.symbols.iter().find(|s| s.name == "find_order").unwrap();
        assert_eq!(func.kind, "function");
        assert!(func.signature.as_deref().unwrap().contains("fn find_order"));
        assert!(func.doc.as_deref().unwrap().contains("주문을 조회한다"));
        assert!(f.calls.iter().any(|c| c.callee == "lookup"));
        Ok(())
    }

    #[test]
    fn extracts_java_spring_shapes() -> Result<()> {
        let src = r#"
package com.example.order;
import org.springframework.web.bind.annotation.GetMapping;

@RestController
public class OrderController {
    @GetMapping("/api/orders/{id}")
    public OrderDto getOrder(Long id) {
        return service.findOne(id);
    }
}
"#;
        let f = extract(SupportedLang::Java, Path::new("A.java"), src)?;
        let names: Vec<_> = f.symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"OrderController"), "got {names:?}");
        assert!(names.contains(&"getOrder"), "got {names:?}");
        assert!(f.calls.iter().any(|c| c.callee == "findOne"));
        assert!(f
            .imports
            .iter()
            .any(|i| i.contains("springframework")), "got {:?}", f.imports);
        Ok(())
    }

    #[test]
    fn extracts_react_hook_shapes() -> Result<()> {
        let src = r#"
import { useState } from "react";

export const useOrder = (id: string) => {
  const [data, setData] = useState(null);
  fetch(`/api/orders/${id}`).then(setData);
  return data;
};

interface OrderDto { id: string }
"#;
        let f = extract(SupportedLang::TypeScript, Path::new("useOrder.ts"), src)?;
        let names: Vec<_> = f.symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"useOrder"), "got {names:?}");
        assert!(names.contains(&"OrderDto"), "got {names:?}");
        assert!(f.calls.iter().any(|c| c.callee == "fetch"));
        assert!(f.imports.iter().any(|i| i == "react"), "got {:?}", f.imports);
        Ok(())
    }

    #[test]
    fn parse_errors_are_reported_not_fatal() -> Result<()> {
        let f = extract(SupportedLang::Rust, Path::new("bad.rs"), "fn broken( {{{")?;
        assert!(f.had_parse_error);
        Ok(())
    }
}

//! 어휘 의미 계층 — 도메인 용어로 물어도 심볼에 착지시킨다 (docs/DESIGN.md 13절)
//!
//! **벡터 임베딩은 넣지 않았다.** 로컬 임베딩 모델이 필요하고, 회사 장비에
//! 모델 파일을 배포하는 것은 별개 문제다. 대신 모델 없이 되는 두 가지를 한다:
//!
//! 1. **식별자 분해** — `deleteComment` 를 "delete comment"로도 색인한다.
//!    자연어 질의가 카멜케이스 식별자에 닿는 대부분의 경우가 이걸로 해결된다.
//! 2. **설정 기반 동의어** — "주문" → "order" 같은 사내·언어 간 매핑.
//!    프레임워크 규칙과 같은 이유로 데이터에 둔다(재빌드 없이 확장).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// `deleteCommentById` → ["delete", "Comment", "By", "Id"]
/// `HTTP_MAX_SIZE` → ["HTTP", "MAX", "SIZE"]
/// `article-service` → ["article", "service"]
pub fn split_identifier(ident: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut prev_lower = false;

    for c in ident.chars() {
        if c == '_' || c == '-' || c == '.' || c == '/' || c == ' ' {
            if !current.is_empty() {
                parts.push(std::mem::take(&mut current));
            }
            prev_lower = false;
            continue;
        }
        // 소문자 → 대문자 경계에서 자른다 (deleteComment)
        if c.is_uppercase() && prev_lower && !current.is_empty() {
            parts.push(std::mem::take(&mut current));
        }
        prev_lower = c.is_lowercase() || c.is_numeric();
        current.push(c);
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts.retain(|p| p.len() > 1);
    parts
}

/// 심볼 이름과 경로를 FTS에 함께 넣을 확장 텍스트로 만든다.
pub fn expand_for_index(name: &str, path: Option<&str>) -> String {
    let mut tokens: Vec<String> = split_identifier(name);
    if let Some(p) = path {
        // 경로의 마지막 두 세그먼트만 — 전체를 넣으면 디렉터리 이름이 지배한다.
        for seg in p.rsplit('/').take(2) {
            tokens.extend(split_identifier(seg));
        }
    }
    tokens.sort();
    tokens.dedup();
    tokens.join(" ")
}

/// 도메인 용어 사전. `nunchi.toml`의 `[semantic]`에서 온다.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Synonyms {
    /// `"주문" = ["order", "orders"]` 처럼 쓴다.
    #[serde(default)]
    pub terms: HashMap<String, Vec<String>>,
}

impl Synonyms {
    /// 질의를 동의어로 확장한다. 원어는 항상 남긴다.
    pub fn expand_query(&self, query: &str) -> String {
        let mut out: Vec<String> = Vec::new();
        for word in query.split_whitespace() {
            out.push(word.to_string());
            let key = word.to_lowercase();
            if let Some(aliases) = self.terms.get(&key) {
                out.extend(aliases.iter().cloned());
            }
            // 질의에 카멜케이스가 들어와도 분해해준다.
            let parts = split_identifier(word);
            if parts.len() > 1 {
                out.extend(parts);
            }
        }
        out.sort();
        out.dedup();
        out.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_camel_snake_and_kebab() {
        assert_eq!(split_identifier("deleteComment"), vec!["delete", "Comment"]);
        assert_eq!(split_identifier("HTTP_MAX_SIZE"), vec!["HTTP", "MAX", "SIZE"]);
        assert_eq!(split_identifier("article-service"), vec!["article", "service"]);
        assert_eq!(
            split_identifier("useArticleCommentQuery"),
            vec!["use", "Article", "Comment", "Query"]
        );
    }

    #[test]
    fn expansion_includes_path_tail_only() {
        let e = expand_for_index("deleteComment", Some("src/main/java/com/x/CommentController.java"));
        assert!(e.contains("delete"));
        assert!(e.contains("Controller"));
        // 상위 디렉터리는 넣지 않는다 — 넣으면 경로가 랭킹을 지배한다.
        assert!(!e.contains("main"));
    }

    #[test]
    fn synonyms_bridge_language_gap() {
        let mut s = Synonyms::default();
        s.terms.insert("주문".into(), vec!["order".into(), "orders".into()]);
        let q = s.expand_query("주문 삭제");
        assert!(q.contains("order"));
        assert!(q.contains("주문"), "원어를 잃으면 안 된다: {q}");
    }

    #[test]
    fn camel_query_is_split() {
        let s = Synonyms::default();
        let q = s.expand_query("deleteComment");
        assert!(q.contains("delete") && q.contains("Comment"));
    }
}

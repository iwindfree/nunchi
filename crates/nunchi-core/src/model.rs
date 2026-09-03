//! 그래프 모델 — docs/DESIGN.md 7절 (노드 18종 / 엣지 19종)

use serde::{Deserialize, Serialize};

/// 안정적인 노드 식별자.
///
/// 형식: `<kind>:<repo>/<path>[#<symbol>]` — 경로는 항상 정규화된 형태
/// (슬래시 구분자, `crate::path::normalize` 참조).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn file(repo: &str, path: &str) -> Self {
        NodeId(format!("file:{repo}/{path}"))
    }
    pub fn repo(repo: &str) -> Self {
        NodeId(format!("repo:{repo}"))
    }
    pub fn symbol(repo: &str, path: &str, symbol: &str) -> Self {
        NodeId(format!("sym:{repo}/{path}#{symbol}"))
    }
    /// 파일 경로를 포함하지 않는 심볼 ID.
    ///
    /// C# `partial class` 는 한 타입이 여러 파일에 흩어진다
    /// (`OrderForm.cs` + `OrderForm.Designer.cs`). 경로를 ID에 넣으면 같은 타입이
    /// 두 노드로 갈라지므로, partial 선언은 경로 없이 식별한다.
    pub fn partial_symbol(repo: &str, symbol: &str) -> Self {
        NodeId(format!("sym:{repo}#{symbol}"))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

macro_rules! str_enum {
    ($name:ident { $($(#[$meta:meta])* $variant:ident => $s:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum $name { $($(#[$meta])* $variant),+ }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $s),+ }
            }
            pub fn parse(s: &str) -> Option<Self> {
                match s { $($s => Some(Self::$variant),)+ _ => None }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

str_enum!(NodeKind {
    Solution => "solution",
    Repo => "repo",
    File => "file",
    Module => "module",
    Symbol => "symbol",
    Test => "test",
    Doc => "doc",
    Commit => "commit",
    Author => "author",
    ExternalDep => "external_dep",
    ConfigKey => "config_key",
    Contract => "contract",
    // 스택별 (docs/DESIGN.md 4·5절)
    Route => "route",
    ApiCall => "api_call",
    Bean => "bean",
    Entity => "entity",
    Table => "table",
    Control => "control",
});

str_enum!(EdgeKind {
    Contains => "contains",
    DefinedIn => "defined_in",
    Imports => "imports",
    Calls => "calls",
    References => "references",
    ExtendsImplements => "extends_implements",
    Tests => "tests",
    Documents => "documents",
    ModifiedBy => "modified_by",
    AuthoredBy => "authored_by",
    CoChangedWith => "co_changed_with",
    DependsOn => "depends_on",
    Exposes => "exposes",
    SharesContract => "shares_contract",
    // 스택별 (docs/DESIGN.md 4·5절)
    CallsApi => "calls_api",
    Injects => "injects",
    PersistsTo => "persists_to",
    Handles => "handles",
    DuplicateOf => "duplicate_of",
});

// 엣지 출처. 2단 속도 인덱싱(docs/DESIGN.md 4·5절)에서 신뢰도 구분에 쓴다.
str_enum!(Provenance {
    /// tree-sitter 빠른 경로 — 파일 저장 시 갱신, 크로스파일 해소는 휴리스틱
    Fast => "fast",
    /// SCIP 정밀 경로 — 빌드 기반, 참조 해소가 정확
    Precise => "precise",
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub name: String,
    pub repo: String,
    /// 저장소 루트 기준 상대 경로 (정규화됨)
    pub path: Option<String>,
    pub span: Option<Span>,
    pub signature: Option<String>,
    pub doc: Option<String>,
    pub lang: Option<String>,
    /// 워킹트리 파일 내용 해시. git blob SHA가 아니다 — CRLF 차이 때문(docs/CONTRIBUTING.md 개발 환경).
    pub content_hash: Option<String>,
    /// 소스 파일의 최종 수정 시각(Unix 초). 랭킹의 recency 항이 쓴다.
    pub mtime: Option<i64>,
    pub attrs: serde_json::Value,
}

impl Node {
    pub fn new(id: NodeId, kind: NodeKind, name: impl Into<String>, repo: impl Into<String>) -> Self {
        Node {
            id,
            kind,
            name: name.into(),
            repo: repo.into(),
            path: None,
            span: None,
            signature: None,
            doc: None,
            lang: None,
            content_hash: None,
            mtime: None,
            attrs: serde_json::Value::Null,
        }
    }

    /// `path:line` 형태의 좌표. 에이전트에게 돌려주는 값의 핵심(docs/DESIGN.md 2절 원칙 1).
    pub fn reference(&self) -> Option<String> {
        let path = self.path.as_ref()?;
        Some(match self.span {
            Some(Span { start_line, end_line }) if end_line > start_line => {
                format!("{path}:{start_line}-{end_line}")
            }
            Some(Span { start_line, .. }) => format!("{path}:{start_line}"),
            None => path.clone(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub src: NodeId,
    pub dst: NodeId,
    pub kind: EdgeKind,
    pub provenance: Provenance,
    /// 0.0~1.0. URL 템플릿 매칭 등 추론된 엣지는 1.0 미만(docs/DESIGN.md 4·5절).
    pub confidence: f32,
    /// 동시변경 결합도 등 가중 엣지용
    pub weight: f32,
}

impl Edge {
    pub fn new(src: NodeId, dst: NodeId, kind: EdgeKind, provenance: Provenance) -> Self {
        Edge { src, dst, kind, provenance, confidence: 1.0, weight: 1.0 }
    }
    pub fn with_confidence(mut self, c: f32) -> Self {
        self.confidence = c;
        self
    }
    pub fn with_weight(mut self, w: f32) -> Self {
        self.weight = w;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Out,
    In,
    Both,
}

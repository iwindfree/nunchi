//! nunchi — 코드베이스 컨텍스트 그래프.
//!
//! 이름은 한국어 **눈치**(말해지지 않은 맥락을 읽는 능력)에서 왔다.
//! 에이전트가 grep으로 볼 수 없는 배선을 읽게 하는 것이 이 도구의 일이다.
//!
//! 설계 문서는 저장소 루트의 `PLAN.md`를 참조한다.

pub mod config;
pub mod extract;
pub mod framework;
pub mod graph;
pub mod index;
pub mod lang;
pub mod model;
pub mod pack;
pub mod path;
pub mod resolve;
pub mod rules;
pub mod store;

pub use config::Config;
pub use model::{Direction, Edge, EdgeKind, Node, NodeId, NodeKind, Provenance, Span};
pub use store::{sqlite::SqliteStore, Store};

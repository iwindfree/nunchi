# 7.1 모듈과 가시성

> **선행 장**: 없습니다.
> **연습문제**: 2개

코드를 파일과 이름 공간으로 나누는 방법입니다.

## 파일이 곧 모듈입니다

```
crates/nunchi-core/src/
├── lib.rs           크레이트의 뿌리입니다
├── model.rs         model 모듈입니다
├── pack.rs          pack 모듈입니다
└── store/
    ├── mod.rs       store 모듈입니다
    └── sqlite.rs    store::sqlite 모듈입니다
```

파일을 만들었다고 자동으로 모듈이 되지는 않습니다. **뿌리에서 선언해야
합니다.**

```rust
// crates/nunchi-core/src/lib.rs 에서
pub mod bench;
pub mod cache;
pub mod config;
pub mod extract;
pub mod framework;
pub mod graph;
pub mod history;
pub mod index;
pub mod lang;
pub mod mapper_xml;
pub mod model;
pub mod pack;
pub mod path;
pub mod resolve;
pub mod rules;
pub mod semantic;
pub mod store;
```

`mod`가 "이 파일을 모듈로 포함한다"는 뜻이고, `pub`이 "밖에서도 쓸 수 있다"는
뜻입니다.

디렉터리를 모듈로 만들려면 그 안에 `mod.rs`를 둡니다. `store/mod.rs`가
`store` 모듈이고, 그 안에서 `pub mod sqlite;`로 하위 모듈을 선언합니다.

## `use`로 이름을 가져옵니다

```rust
use crate::model::{Edge, EdgeKind, Node, NodeId};
use std::collections::HashMap;
```

`use`가 없으면 매번 전체 경로를 적어야 합니다.

```rust
let id = crate::model::NodeId::file("api", "a.rs");    // 길어집니다
let id = NodeId::file("api", "a.rs");                   // use 가 있으면 짧습니다
```

경로 앞에 오는 이름의 뜻입니다.

| 접두 | 뜻 |
|---|---|
| `crate::` | 지금 크레이트의 뿌리부터 |
| `super::` | 한 단계 위 모듈 |
| `self::` | 지금 모듈 |
| 이름으로 시작 | 외부 크레이트 |

```rust
// crates/nunchi-core/src/store/sqlite.rs 에서
use super::{RankOpts, Ranked, SearchHit, Store};    // store/mod.rs 에서 가져옵니다
use crate::model::*;                                 // 크레이트 뿌리에서 갑니다
use rusqlite::{params, Connection};                  // 외부 크레이트입니다
```

`*`는 그 모듈의 공개된 것을 전부 가져옵니다. 편하지만 어디서 온 이름인지
알기 어려워지므로 되도록 쓰지 않습니다. `model`처럼 타입이 많고 자주 쓰는
모듈에만 씁니다.

## `pub`이 없으면 밖에서 못 씁니다

```rust
pub struct Node {         // 타입은 공개입니다
    pub id: NodeId,       // 이 필드도 공개입니다
    name: String,         // 이 필드는 비공개입니다
}
```

**타입과 필드에 각각 붙여야 합니다.** 타입만 `pub`이고 필드가 아니면 밖에서
필드를 읽을 수 없습니다.

nunchi의 구조체는 대부분 필드까지 공개입니다. 라이브러리가 아니라 애플리케이션
내부에서만 쓰기 때문입니다. 라이브러리라면 필드를 감추고 메서드로만
접근하게 만드는 편이 낫습니다.

함수도 마찬가지입니다.

```rust
// crates/nunchi-core/src/index.rs 에서
pub fn index_all(config: &Config, store: &mut SqliteStore) -> Result<IndexStats>   // 공개
fn scan_repo(...) -> Result<Vec<String>>                                           // 내부용
fn repo_name(root: &Path) -> String                                                // 내부용
```

`index_all`만 밖에서 부를 수 있습니다. 나머지는 이 모듈 안에서만 쓰입니다.
**공개 범위를 좁게 두면 나중에 고칠 때 영향 범위가 작아집니다.**

## 크레이트가 두 개입니다

```
crates/
├── nunchi-core/     라이브러리입니다. 모든 로직이 여기 있습니다
└── nunchi-cli/      실행 파일입니다. 얇은 진입점입니다
```

`nunchi-cli`가 `nunchi-core`를 씁니다.

```rust
// crates/nunchi-cli/src/main.rs 에서
use nunchi_core::config::{Config, IndexConfig, RankWeights, Solution, CONFIG_FILE};
use nunchi_core::store::Store;
use nunchi_core::{index, lang, SqliteStore};
```

크레이트 이름에서 `-`가 `_`로 바뀝니다. `nunchi-core`가 코드에서는
`nunchi_core`입니다.

**나눈 이유가 있습니다.** 로직이 라이브러리에 있으면 테스트하기 쉽고, MCP
서버와 CLI와 TUI가 같은 코드를 직접 부를 수 있습니다. 로직을 `main.rs`에
두면 그렇게 할 수 없습니다.

## 자주 쓰는 것을 뿌리에서 다시 내보냅니다

```rust
// crates/nunchi-core/src/lib.rs 에서
pub use config::Config;
pub use model::{Direction, Edge, EdgeKind, Node, NodeId, NodeKind, Provenance, Span};
pub use store::{sqlite::SqliteStore, Store};
```

`pub use`는 "이 이름을 여기서도 쓸 수 있게 한다"는 뜻입니다. 덕분에 쓰는
쪽이 짧아집니다.

```rust
use nunchi_core::Node;                    // 이렇게 쓸 수 있습니다
use nunchi_core::model::Node;             // 이것도 됩니다
```

## 연습문제

### 문제 1 [고치기]

```bash
cd book/exercises
cargo test -p ex_07_01_a
```

`pub`이 빠져서 밖에서 쓸 수 없는 코드를 고치는 문제입니다.

### 문제 2 [쓰기]

```bash
cargo test -p ex_07_01_b
```

모듈을 나누고 `use`로 가져오는 문제입니다.

## 정리

파일이 곧 모듈이지만 뿌리에서 `mod`로 선언해야 포함됩니다. 디렉터리를 모듈로
만들려면 `mod.rs`를 둡니다.

`pub`이 없으면 밖에서 쓸 수 없으며, 타입과 필드에 각각 붙여야 합니다. 공개
범위를 좁게 두면 고칠 때 영향 범위가 작아집니다.

nunchi는 라이브러리와 실행 파일 두 크레이트로 나뉘어 있습니다. 로직이
라이브러리에 있어야 MCP 서버와 CLI와 TUI가 같은 코드를 쓸 수 있습니다.

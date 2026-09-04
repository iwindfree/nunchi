# 5.1 `impl`, 연관 함수와 메서드

> **선행 장**: [0.4 구조체, 열거형, 튜플](00-4-data.md), [1.3 빌림](01-3-borrow.md)
> **연습문제**: 2개

구조체에 동작을 붙이는 방법입니다. nunchi에 `impl` 블록이 25개 있습니다.

## 데이터와 동작이 분리되어 있습니다

다른 언어의 클래스는 필드와 메서드를 한 곳에 적습니다. Rust는 나눕니다.

```rust
// 데이터를 정의합니다
pub struct NodeId(pub String);

// 동작을 붙입니다
impl NodeId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

`impl`은 implementation의 줄임말입니다. 한 타입에 `impl` 블록을 여러 개 둘
수도 있습니다.

## 두 종류가 있습니다

`self`를 받는지가 기준입니다.

```rust
impl NodeId {
    // 연관 함수: self 가 없습니다. 새 값을 만들 때 씁니다
    pub fn file(repo: &str, path: &str) -> Self {
        NodeId(format!("file:{repo}/{path}"))
    }

    // 메서드: self 가 있습니다. 이미 있는 값에 대해 동작합니다
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

부르는 방법이 다릅니다.

```rust
let id = NodeId::file("api", "src/A.java");    // 연관 함수는 :: 로 부릅니다
let s = id.as_str();                            // 메서드는 . 으로 부릅니다
```

`Self`는 대문자로 시작하며 "이 `impl` 블록이 붙은 타입"을 뜻합니다.
`NodeId`라고 적어도 되지만 `Self`가 짧고, 타입 이름이 바뀌어도 고칠 필요가
없습니다.

## `self`를 받는 세 가지 방식

여기가 [1.3장](01-3-borrow.md)의 빌림과 만나는 지점입니다.

| 표기 | 뜻 | 언제 |
|---|---|---|
| `&self` | 빌려서 읽습니다 | 값을 바꾸지 않을 때 |
| `&mut self` | 빌려서 바꿉니다 | 값을 바꿀 때 |
| `self` | 소유권을 가져갑니다 | 값을 소비하거나 변형할 때 |

nunchi에서 세 가지가 모두 쓰입니다.

```rust
// crates/nunchi-core/src/store/sqlite.rs 에서
pub fn count_nodes(&self) -> Result<i64>              // 읽기만 합니다
pub fn upsert_nodes(&mut self, nodes: &[Node])        // 데이터베이스를 바꿉니다
```

```rust
// crates/nunchi-core/src/model.rs 에서
pub fn with_confidence(mut self, c: f32) -> Self {
    self.confidence = c;
    self
}
```

세 번째가 특이합니다. `self`를 통째로 받아서 필드를 바꾼 다음 다시 돌려줍니다.
이렇게 하면 점으로 이어 쓸 수 있습니다.

```rust
Edge::new(src, dst, EdgeKind::Calls, Provenance::Fast)
    .with_confidence(0.8)
```

이 방식을 빌더(builder)라고 부릅니다. 필드가 많은 구조체를 만들 때 필수가
아닌 값만 골라 지정할 수 있어 편합니다.

`mut self`에서 `mut`은 "받은 값을 이 함수 안에서 바꾸겠다"는 뜻입니다.
소유권을 가져왔으므로 마음대로 바꿔도 됩니다.

## 이 프로젝트에서는

`NodeId`의 `impl` 블록 전체를 봅니다.

```rust
// crates/nunchi-core/src/model.rs 에서
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
    pub fn partial_symbol(repo: &str, symbol: &str) -> Self {
        NodeId(format!("sym:{repo}#{symbol}"))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

연관 함수 네 개와 메서드 하나입니다.

**이 형식들을 `impl` 안에 모아 둔 이유가 있습니다.** 코드 여기저기서
`format!("file:{}/{}", repo, path)`라고 직접 쓰면, 형식을 바꿀 때 모든 자리를
찾아야 합니다. 한곳에 모아 두면 여기만 고치면 됩니다.

그리고 `partial_symbol`이 경로를 빼는 이유가 주석에 적혀 있습니다.
C#의 `partial class`는 한 타입이 여러 파일에 흩어지므로 경로를 넣으면 노드가
쪼개집니다.

## 이름이 겹쳐도 됩니다

타입마다 자기 `impl`을 가지므로 이름이 겹쳐도 문제가 없습니다.

```rust
store.clear();      // SqliteStore 의 clear 입니다
names.clear();      // Vec 의 clear 입니다
```

어느 타입의 메서드인지는 앞에 오는 값이 정합니다.

## 연습문제

### 문제 1 [쓰기]

```bash
cd book/exercises
cargo test -p ex_05_01_a
```

연관 함수와 메서드를 구분해서 작성하는 문제입니다.

### 문제 2 [고치기]

```bash
cargo test -p ex_05_01_b
```

`&self`와 `&mut self`를 잘못 쓴 코드를 고치는 문제입니다.

## 정리

`impl` 블록으로 타입에 동작을 붙입니다. `self`가 없으면 연관 함수이며 `::`로
부르고, 있으면 메서드이며 `.`으로 부릅니다.

`self`를 받는 방식은 빌림 규칙을 그대로 따릅니다. 읽기만 하면 `&self`,
바꾸면 `&mut self`, 소유권이 필요하면 `self`입니다.

`self`를 받아 다시 돌려주면 점으로 이어 쓸 수 있습니다. 이것을 빌더라고
부릅니다.

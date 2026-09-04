# 5.2 트레이트

> **선행 장**: [5.1 `impl`, 연관 함수와 메서드](05-1-impl.md)
> **연습문제**: 2개

트레이트는 "이 동작들을 할 수 있다"는 약속입니다. 다른 언어의 인터페이스와
비슷합니다.

nunchi에는 트레이트 정의가 **하나뿐입니다.** 그 하나가 설계에서 중요한 자리를
차지하므로 그것으로 설명합니다.

## 정의하고 구현합니다

```rust
// 약속을 정의합니다
pub trait Store {
    fn upsert_nodes(&mut self, nodes: &[Node]) -> Result<usize>;
    fn upsert_edges(&mut self, edges: &[Edge]) -> Result<usize>;
    fn neighbors(&self, id: &NodeId, kinds: &[EdgeKind], dir: Direction, depth: u32)
        -> Result<Vec<Node>>;
    fn paths(&self, from: &NodeId, to: &NodeId, max_depth: u32) -> Result<Vec<Vec<NodeId>>>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>>;
    fn rank(&self, seeds: &[NodeId], opts: &RankOpts) -> Result<Vec<Ranked>>;
}
```

본문이 없고 서명만 있습니다. **무엇을 할 수 있어야 하는지만 적습니다.**

```rust
// 특정 타입이 그 약속을 지킵니다
impl Store for SqliteStore {
    fn upsert_nodes(&mut self, nodes: &[Node]) -> Result<usize> {
        // 실제 구현입니다
    }
    // 나머지 다섯 개도 모두 구현해야 합니다
}
```

`impl 트레이트 for 타입` 형태입니다. **여섯 개를 모두 구현하지 않으면
컴파일되지 않습니다.**

## 왜 이 하나가 중요한가

nunchi는 SQLite를 씁니다. 그런데 처음부터 그 결정을 되돌릴 수 있게 만들어
두었습니다.

설계 문서에 이렇게 적혀 있습니다.

> 저장 계층은 여섯 개 메서드 뒤에 있으므로 교체 비용이 하루를 넘지 않습니다.

만약 LadybugDB 같은 그래프 데이터베이스로 옮기고 싶어지면, `Store`를 구현하는
새 타입을 하나 만들면 됩니다. 나머지 코드는 손대지 않습니다.

**그래서 이 트레이트에 메서드를 추가하지 않는 것이 중요합니다.** 여섯 개가
열두 개가 되면 교체 비용이 두 배가 됩니다. 기여 안내서에 이렇게 적어 두었습니다.

> 메서드를 늘리기 전에 다시 생각하십시오.

## 트레이트 밖의 메서드

`SqliteStore`에는 트레이트에 없는 메서드도 많습니다.

```rust
// crates/nunchi-core/src/store/sqlite.rs 에서
impl SqliteStore {
    pub fn open(path: &Path) -> Result<Self> { }
    pub fn count_nodes(&self) -> Result<i64> { }
    pub fn files_by_lang(&self) -> Result<Vec<(String, i64)>> { }
    pub fn prune_missing_files(&mut self, ...) -> Result<usize> { }
}

impl Store for SqliteStore {
    // 여섯 개만 여기 있습니다
}
```

`impl` 블록이 두 개입니다. 하나는 이 타입 고유의 동작이고, 다른 하나는
트레이트 약속입니다.

**이 구분이 실제로 값을 합니다.** 다른 데이터베이스로 옮길 때, 트레이트에 있는
여섯 개는 반드시 구현해야 하고 나머지는 필요한 것만 옮기면 됩니다. 무엇이
필수인지 코드가 알려 줍니다.

## 표준 라이브러리 트레이트

직접 정의하지 않아도 이미 쓰고 있는 트레이트가 많습니다.

| 트레이트 | 하는 일 |
|---|---|
| `Clone` | `.clone()`을 쓸 수 있습니다 |
| `Copy` | 복사가 자동으로 일어납니다 |
| `Debug` | `{:?}`로 출력할 수 있습니다 |
| `Display` | `{}`로 출력할 수 있습니다 |
| `PartialEq` | `==`로 비교할 수 있습니다 |
| `Default` | `Default::default()`로 기본값을 만듭니다 |
| `From`, `Into` | 타입을 변환합니다 |
| `Iterator` | 값을 하나씩 꺼내 줍니다 |

이 중 대부분은 직접 구현하지 않고 `#[derive]`로 자동 생성합니다.
[5.4장](05-4-derive.md)에서 다룹니다.

`Display`는 직접 구현해야 합니다. 어떻게 보여 줄지는 사람이 정해야 하기
때문입니다.

```rust
// crates/nunchi-core/src/model.rs 에서
impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
```

이렇게 해 두면 `println!("{}", node_id)`가 됩니다.

## nunchi에 트레이트가 하나뿐인 이유

트레이트는 **여러 타입이 같은 약속을 지켜야 할 때** 값을 합니다. 그런 상황이
많지 않으면 굳이 만들 이유가 없습니다.

nunchi에서 그런 상황은 저장 계층 하나뿐이었습니다. 추출기는 언어마다
다르지만 `SupportedLang` 열거형으로 갈라내면 충분했고, 프레임워크 규칙은
트레이트가 아니라 **데이터**로 두는 편이 나았습니다.

> 규칙을 트레이트로 만들었다면 새 프레임워크를 지원할 때마다 코드를 고치고
> 다시 빌드해야 합니다. 데이터로 두면 설정 파일만 고치면 됩니다.

**추상화를 미리 만들지 않는 것**도 설계입니다. 필요해지면 그때 만들어도
늦지 않습니다.

## 연습문제

### 문제 1 [쓰기]

```bash
cd book/exercises
cargo test -p ex_05_02_a
```

트레이트를 정의하고 구현하는 문제입니다.

### 문제 2 [고치기]

```bash
cargo test -p ex_05_02_b
```

`Display`를 구현하는 문제입니다.

## 정리

트레이트는 "이 동작들을 할 수 있다"는 약속이며, `impl 트레이트 for 타입`으로
구현합니다. 약속한 메서드를 모두 구현하지 않으면 컴파일되지 않습니다.

nunchi의 `Store` 트레이트는 저장 계층을 교체 가능하게 만드는 장치입니다.
여섯 개로 좁게 유지하는 것이 그 장치가 동작하는 조건입니다.

여러 타입이 같은 약속을 지켜야 할 때만 트레이트를 만듭니다. 추상화를 미리
만들지 않는 것도 설계입니다.

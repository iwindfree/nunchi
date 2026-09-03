# 0.4 구조체, 열거형, 튜플

> **선행 장**: [0.3 타입 표기 읽는 법](00-3-types.md)
> **연습문제**: 2개

데이터를 담는 방법이 세 가지 있습니다. 이 셋을 알면 nunchi의 모든 데이터
구조를 읽을 수 있습니다.

## 튜플

값 여러 개를 괄호로 묶은 것입니다. 이름이 없습니다.

```rust
let pair = (3, "hello");
let first = pair.0;        // 3
let second = pair.1;       // "hello"
```

타입은 `(i32, &str)`로 적습니다. nunchi 코드에 튜플이 183번 나옵니다. 주로
함수가 값 두세 개를 한꺼번에 돌려줄 때 씁니다.

```rust
// crates/nunchi-core/src/framework.rs 에서
pub fn tables_in_sql(sql: &str) -> Vec<(String, String)> {
```

`(String, String)`은 테이블 이름과 동작(`select`, `insert` 등)의 짝입니다.

튜플에서 값을 꺼낼 때는 이렇게 한 번에 풀 수도 있습니다.

```rust
let (table, verb) = pair;
```

이것을 구조 분해(destructuring)라고 부릅니다. 3부에서 더 다룹니다.

**튜플은 값이 두세 개이고 의미가 명백할 때만 씁니다.** 네 개가 넘거나 각
값이 무엇인지 헷갈리면 구조체를 씁니다. `.0`, `.1`, `.2`, `.3`을 읽는 사람은
그것이 무엇인지 알 수 없기 때문입니다.

## 구조체

값에 이름을 붙여 묶은 것입니다.

```rust
pub struct Span {
    pub start_line: u32,
    pub end_line: u32,
}
```

만들 때는 이름과 값을 함께 적습니다.

```rust
let span = Span { start_line: 10, end_line: 25 };
println!("{}", span.start_line);      // 10
```

`pub`은 다른 모듈에서 쓸 수 있게 공개한다는 뜻입니다. 7.1장에서 다룹니다.

### 튜플 구조체

이름 없는 필드를 가진 구조체도 있습니다. 값 하나를 감쌀 때 씁니다.

```rust
pub struct NodeId(pub String);

let id = NodeId("file:api/A.java".to_string());
println!("{}", id.0);                  // 안쪽 문자열을 꺼냅니다
```

`NodeId`는 그냥 `String`을 감싼 것인데, 왜 감싸는지 이유가 있습니다. **함수가
`String`을 받는다면 아무 문자열이나 넘길 수 있지만, `NodeId`를 받는다면
노드 식별자만 넘길 수 있습니다.** 컴파일러가 실수를 막아 줍니다.

```rust
fn get_node(id: &NodeId) -> Option<Node>    // 좋습니다
fn get_node(id: &str) -> Option<Node>       // 아무 문자열이나 들어옵니다
```

## 열거형

**여러 가능성 중 하나**를 나타냅니다. 다른 언어의 enum과 비슷하지만 훨씬
강력합니다.

```rust
pub enum Direction {
    Out,
    In,
    Both,
}
```

`Direction`은 이 세 값 중 정확히 하나입니다. 네 번째는 없습니다.

### 값을 품는 열거형

각 가능성이 서로 다른 데이터를 가질 수 있습니다. 이것이 다른 언어의 enum과
크게 다른 점입니다.

```rust
enum Verified {
    Fresh(String),      // 파일 내용을 품습니다
    Unknown,            // 아무것도 품지 않습니다
    Stale,              // 아무것도 품지 않습니다
}
```

nunchi의 실제 코드입니다. 팩을 만들 때 파일을 검증한 결과를 나타냅니다.
`Fresh`일 때만 파일 내용이 있고, 나머지 두 경우에는 없습니다. **없는 경우에
내용을 읽으려는 코드는 컴파일되지 않습니다.** 이것이 열거형의 값어치입니다.

`Option`과 `Result`도 열거형입니다.

```rust
enum Option<T> {
    Some(T),        // 값이 있습니다
    None,           // 값이 없습니다
}
```

2부에서 자세히 다룹니다. 지금은 `Option`과 `Result`가 특별한 문법이 아니라
그냥 열거형이라는 사실만 알아 두시면 됩니다.

## 셋을 언제 쓰는가

| 상황 | 선택 |
|---|---|
| 값 두세 개를 임시로 묶는다 | 튜플 |
| 값들이 각자 이름을 가져야 한다 | 구조체 |
| 값 하나를 감싸 타입을 구분한다 | 튜플 구조체 |
| 여러 가능성 중 하나다 | 열거형 |

## 이 프로젝트에서는

nunchi의 핵심 데이터 구조를 하나 읽어 보겠습니다.

```rust
// crates/nunchi-core/src/model.rs 에서
pub struct Edge {
    pub src: NodeId,
    pub dst: NodeId,
    pub kind: EdgeKind,
    pub provenance: Provenance,
    pub confidence: f32,
    pub weight: f32,
}
```

- `src`와 `dst`는 엣지의 양 끝 노드입니다. 튜플 구조체 `NodeId`입니다.
- `kind`는 엣지 종류이며 열거형입니다. `Calls`, `Imports` 등 19가지 중 하나입니다.
- `provenance`도 열거형이며 `Fast`와 `Precise` 둘 중 하나입니다.
- `confidence`와 `weight`는 실수입니다.

`kind`를 문자열로 두지 않고 열거형으로 둔 이유가 있습니다. 문자열이면
`"cals"`처럼 잘못 적어도 컴파일이 되지만, 열거형이면 없는 값을 쓸 수 없습니다.

## 연습문제

### 문제 1 [읽기]

아래 코드에서 `Verified::Unknown`인 경우에 `text`를 읽으려고 하면 어떻게
됩니까?

```rust
enum Verified {
    Fresh(String),
    Unknown,
    Stale,
}
```

<details>
<summary>정답 보기</summary>

애초에 읽을 방법이 없습니다.

`Unknown`은 데이터를 품지 않으므로 꺼낼 것이 없습니다. Rust에서는 열거형의
값을 꺼내려면 `match`로 어떤 경우인지 먼저 확인해야 하는데(3.1장), 그 과정에서
`Unknown`인 경우를 반드시 처리하게 됩니다.

이것이 열거형이 안전한 이유입니다. 다른 언어에서 "파일 내용이 있을 수도 있고
없을 수도 있는 값"을 다루면 없는 경우를 잊기 쉽지만, Rust에서는 컴파일러가
잊게 놓아두지 않습니다.

</details>

### 문제 2 [쓰기]

```bash
cd book/exercises
cargo test -p ex_00_04_a
```

구조체와 열거형을 정의하는 문제입니다.

## 정리

튜플은 이름 없이 값을 묶고, 구조체는 이름을 붙여 묶으며, 열거형은 여러
가능성 중 하나를 나타냅니다. 튜플 구조체는 값 하나를 감싸 타입을 구분하는 데
씁니다. `Option`과 `Result`도 열거형입니다.

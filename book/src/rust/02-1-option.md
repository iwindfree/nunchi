# 2.1 `Option<T>`

> **선행 장**: [0.4 구조체, 열거형, 튜플](00-4-data.md), [1.3 빌림](01-3-borrow.md)
> **연습문제**: 3개

nunchi 코드에 `Option`이 100번 나옵니다. 값이 없을 수도 있는 상황을 다루는
방법입니다.

## 다른 언어의 null 과 무엇이 다른가

대부분의 언어에는 "값이 없음"을 나타내는 특별한 값이 있습니다. Java의 `null`,
Python의 `None`, JavaScript의 `undefined`가 그렇습니다.

문제는 **그 값이 어디에나 들어갈 수 있다는 점**입니다. 문자열을 받는 함수에
`null`이 들어올 수 있고, 그것을 확인하지 않으면 실행 중에 오류가 납니다.

Rust에는 `null`이 없습니다. 대신 값이 없을 수 있다는 사실을 **타입에 적습니다.**

```rust
fn find(name: &str) -> Option<String>
```

이 함수는 문자열을 돌려줄 수도 있고 안 돌려줄 수도 있습니다. 서명만 보고 알 수
있습니다. 반대로 이런 함수는 반드시 값을 돌려줍니다.

```rust
fn normalize(path: &str) -> String
```

**없을 수 있는 값과 반드시 있는 값이 서로 다른 타입입니다.** 그래서 확인을
잊을 수가 없습니다.

## `Option`은 그냥 열거형입니다

특별한 문법이 아니라 표준 라이브러리에 정의된 열거형입니다.

```rust
enum Option<T> {
    Some(T),        // 값이 있으며 그 값을 품고 있습니다
    None,           // 값이 없습니다
}
```

0.4장에서 본 "값을 품는 열거형"과 같은 구조입니다.

```rust
let found: Option<String> = Some("OrderService".to_string());
let missing: Option<String> = None;
```

`Some`과 `None`은 어디서나 바로 쓸 수 있습니다. `Option::Some`이라고 적지
않아도 됩니다.

## 값을 꺼내는 방법

`Option` 안의 값을 쓰려면 먼저 있는지 확인해야 합니다. 방법이 여러 가지입니다.

### `unwrap_or` 계열

없을 때 대신 쓸 값을 지정합니다. nunchi 코드에 89번 나옵니다.

```rust
let name = found.unwrap_or_default();              // 없으면 빈 문자열입니다
let name = found.unwrap_or("unknown".to_string()); // 없으면 이 값을 씁니다
let name = found.unwrap_or_else(|| compute());     // 없을 때만 함수를 부릅니다
```

`unwrap_or`와 `unwrap_or_else`의 차이가 중요합니다. `unwrap_or`는 대체값을
**항상 미리 만듭니다.** 값이 있어도 만듭니다. 대체값을 만드는 비용이 크면
`unwrap_or_else`를 써서 필요할 때만 만들게 해야 합니다.

```rust
// crates/nunchi-cli/src/main.rs 에서
let lang = language_of(path).unwrap_or_else(|| "unknown".to_string());
```

여기서는 `to_string()`이 힙 할당을 하므로 `unwrap_or_else`가 맞습니다.

### `unwrap`과 `expect`

값이 없으면 프로그램을 멈춥니다.

```rust
let name = found.unwrap();                    // 없으면 멈춥니다
let name = found.expect("이름이 있어야 합니다"); // 없으면 이 메시지와 함께 멈춥니다
```

**실제 코드에서는 되도록 쓰지 않습니다.** nunchi에는 27번 나오는데 대부분
테스트 안입니다. 테스트에서는 값이 없으면 실패해야 하므로 적절합니다.

본체 코드에서 쓰는 경우는 "여기서는 값이 반드시 있다"는 사실을 코드 구조로
보장할 수 있을 때뿐입니다.

```rust
// crates/nunchi-core/src/store/sqlite.rs 에서
let tail = path.last().expect("경로는 비어 있지 않다");
```

바로 위에서 경로에 값을 넣었으므로 비어 있을 수 없습니다. `expect`의 메시지가
그 이유를 설명합니다.

### `?` 연산자

없으면 함수를 즉시 끝냅니다. [2.3장](02-3-question-mark.md)에서 다룹니다.

```rust
fn extension(path: &str) -> Option<String> {
    let (_, ext) = path.rsplit_once('.')?;    // 없으면 None 을 돌려주고 끝냅니다
    Some(ext.to_string())
}
```

### `if let`과 `match`

3부에서 다룹니다.

## 자주 쓰는 변환

`Option`에는 편리한 메서드가 많습니다. nunchi에서 자주 나오는 것들입니다.

| 메서드 | 하는 일 |
|---|---|
| `.map(f)` | 값이 있으면 `f`를 적용합니다. 없으면 그대로 `None`입니다 |
| `.and_then(f)` | `.map`과 비슷하나 `f`가 `Option`을 돌려줍니다 |
| `.filter(pred)` | 조건에 맞지 않으면 `None`으로 바꿉니다 |
| `.as_deref()` | `Option<String>`을 `Option<&str>`로 바꿉니다 |
| `.is_some()`, `.is_none()` | 있는지 없는지만 확인합니다 |
| `.ok_or(err)` | `Option`을 `Result`로 바꿉니다 |

`.map`을 예로 봅니다.

```rust
let ext: Option<String> = path.rsplit_once('.').map(|(_, e)| e.to_string());
```

`rsplit_once`는 `Option<(&str, &str)>`을 돌려줍니다. 값이 있으면 뒷부분만
꺼내 `String`으로 바꾸고, 없으면 그대로 `None`입니다. **없는 경우를 따로
처리하지 않아도 됩니다.**

`.as_deref()`는 nunchi에 자주 나옵니다.

```rust
// crates/nunchi-core/src/pack.rs 에서
let (Some(rel), Some(root)) = (node.path.as_deref(), roots.get(&node.repo)) else {
```

`node.path`는 `Option<String>`인데 읽기만 하면 되므로 `Option<&str>`로
바꿉니다. 복사를 피하는 방법입니다.

## 이 프로젝트에서는

`Option`이 쓰이는 전형적인 자리를 봅니다.

```rust
// crates/nunchi-core/src/model.rs 에서
pub struct Node {
    pub path: Option<String>,      // 파일에 속하지 않는 노드도 있습니다
    pub span: Option<Span>,        // 위치가 없는 노드도 있습니다
    pub signature: Option<String>,
    pub doc: Option<String>,
    pub mtime: Option<i64>,
}
```

`Commit`이나 `Author` 노드에는 경로가 없습니다. 반면 `File`이나 `Symbol`
노드에는 있습니다. 하나의 구조체가 두 경우를 모두 담아야 하므로 `Option`을
씁니다.

이 설계 덕분에 코드에서 실수를 막을 수 있습니다.

```rust
// crates/nunchi-core/src/store/sqlite.rs 에서
"SELECT id FROM nodes
 WHERE repo = ?1 AND path IS NOT NULL
   AND key NOT IN (SELECT k FROM seen_keys)",
```

사라진 파일의 노드를 지우는 질의입니다. `path IS NOT NULL` 조건이 있으므로
경로가 없는 `Commit`과 `Author`는 건드리지 않습니다. 타입에 `Option`이
있으므로 이 구분이 필요하다는 사실을 코드를 쓸 때 자연스럽게 인식하게 됩니다.

## 연습문제

### 문제 1 [읽기]

아래 두 코드의 차이는 무엇입니까? 어느 쪽이 나은지 판단하십시오.

```rust
// (가)
let name = maybe_name.unwrap_or(expensive_default());

// (나)
let name = maybe_name.unwrap_or_else(|| expensive_default());
```

<details>
<summary>정답 보기</summary>

(나)가 낫습니다.

(가)는 `maybe_name`에 값이 있어도 `expensive_default()`를 **항상**
호출합니다. 인자를 넘기려면 먼저 만들어야 하기 때문입니다.

(나)는 함수를 넘기므로 값이 없을 때만 실행됩니다.

대체값이 상수이거나 만드는 비용이 없으면 (가)를 써도 됩니다.
`unwrap_or(0)`처럼 말입니다.

</details>

### 문제 2 [고치기]

```bash
cd book/exercises
cargo test -p ex_02_01_a
```

`Option`을 반환하도록 서명을 고치는 문제입니다.

### 문제 3 [쓰기]

```bash
cargo test -p ex_02_01_b
```

`.map`과 `.unwrap_or_else`를 써서 값을 꺼내는 문제입니다.

## 정리

Rust에는 `null`이 없고 대신 `Option<T>`를 씁니다. 값이 없을 수 있다는 사실이
타입에 적히므로 확인을 잊을 수 없습니다.

`Option`은 특별한 문법이 아니라 `Some`과 `None` 두 값을 갖는 열거형입니다.

값을 꺼낼 때는 `unwrap_or` 계열이나 `?` 연산자, `match`를 씁니다. `unwrap`은
실제 코드에서 되도록 피하고 테스트에서만 씁니다.

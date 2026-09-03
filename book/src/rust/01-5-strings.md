# 1.5 문자열 세 종류

> **선행 장**: [1.3 빌림 `&`와 `&mut`](01-3-borrow.md)
> **연습문제**: 4개

Rust를 처음 쓰면 문자열에서 자주 막힙니다. 종류가 여러 개이고 서로 바꿔 써야
하기 때문입니다. 이 장에서 정리합니다.

## 두 가지가 기본입니다

| 타입 | 뜻 | 힙을 쓰는가 |
|---|---|---|
| `String` | 소유한 문자열 | 씁니다 |
| `&str` | 빌린 문자열 조각 | 쓰지 않습니다 |

관계는 `Vec<T>`와 `&[T]`의 관계와 같습니다. 하나는 소유하고 하나는 빌립니다.

```rust
let owned: String = String::from("OrderService");
let borrowed: &str = &owned;        // 빌립니다
```

코드에 큰따옴표로 직접 쓴 문자열은 `&str`입니다.

```rust
let name = "OrderService";          // 타입은 &str 입니다
```

이것이 `String`이 아닌 이유가 있습니다. 이 문자는 프로그램 실행 파일 안에
들어 있고 실행 내내 그 자리에 있습니다. 힙에 새로 만들 필요가 없으므로 빌려서
쓰기만 합니다.

## 서로 바꾸는 방법

```rust
// &str 에서 String 으로
let owned = "hello".to_string();
let owned = String::from("hello");
let owned = "hello".to_owned();
```

셋 다 같은 일을 합니다. nunchi 코드에는 `.to_string()`이 가장 많이 나옵니다.

```rust
// String 에서 &str 로
let borrowed: &str = &owned;
let borrowed: &str = owned.as_str();
```

`&owned`만으로 되는 이유는 Rust가 필요할 때 자동으로 변환해 주기 때문입니다.

## 어느 쪽을 써야 하는가

**함수 매개변수에는 `&str`을 씁니다.**

```rust
// 좋습니다
fn make_id(path: &str) -> String

// 이유 없이 제약을 겁니다
fn make_id(path: String) -> String
```

`&str`을 받으면 `String`도 넘길 수 있고 코드에 직접 쓴 문자열도 넘길 수
있습니다. `String`을 받으면 호출하는 쪽에서 `.to_string()`을 불러야 하고,
그것은 불필요한 힙 할당입니다.

```rust
fn make_id(path: &str) -> String { format!("file:{path}") }

make_id("src/main.rs");           // 됩니다
make_id(&some_string);            // 됩니다
```

**구조체 필드에는 `String`을 씁니다.**

```rust
// crates/nunchi-core/src/model.rs 에서
pub struct Node {
    pub name: String,
    pub repo: String,
    pub path: Option<String>,
    // ...
}
```

`&str`을 넣으면 수명 표기가 필요해지고, 그 구조체를 함수 밖으로 돌려주기가
어려워집니다. [1.6장](01-6-lifetimes.md)에서 다룹니다.

**함수 반환값에는 대개 `String`을 씁니다.** 함수 안에서 만든 문자열은 함수가
끝나면 사라지므로 빌려서 돌려줄 수 없습니다.

## `format!`

문자열을 조립할 때 씁니다. nunchi 코드에 103번 나옵니다.

```rust
let id = format!("file:{}/{}", repo, path);
```

중괄호에 변수 이름을 직접 넣을 수도 있습니다.

```rust
let id = format!("file:{repo}/{path}");
```

`format!`은 **읽기만 합니다.** 그래서 그 뒤에도 원본을 쓸 수 있습니다.
[1.4장](01-4-clone.md)에서 본 요령이 여기서 나옵니다.

```rust
let msg = format!("added {}", name);   // name 을 빌려서 읽습니다
names.push(name);                      // 그래서 여기서 넘길 수 있습니다
```

## 이 프로젝트에서는

경로를 다루는 함수를 보겠습니다.

```rust
// crates/nunchi-core/src/path.rs 에서
pub fn normalize(path: &Path) -> String {
    let s = path.to_string_lossy();
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    s.replace('\\', "/")
}
```

세 줄에서 문자열이 세 번 바뀝니다.

- `path.to_string_lossy()`는 경로를 문자열로 바꿉니다. 반환 타입이
  `Cow<str>`인데, 이것은 "복사했을 수도 있고 빌렸을 수도 있다"는 뜻입니다.
  대부분의 경우 빌린 것이라 복사가 없습니다.
- `strip_prefix`는 접두를 떼고 `&str`을 돌려줍니다. 접두가 없으면 `None`이므로
  `unwrap_or(&s)`로 원본을 그대로 씁니다.
- `replace`는 새 `String`을 만듭니다. 여기서 처음으로 힙 할당이 일어납니다.

**필요할 때까지 복사를 미루는 방식**입니다. 앞의 두 단계는 빌리기만 하고,
실제로 내용을 바꿔야 하는 마지막 단계에서만 새로 만듭니다.

비교 전용 함수도 봅니다.

```rust
pub fn compare_key(normalized: &str) -> String {
    normalized.to_lowercase()
}
```

`&str`을 받아 `String`을 돌려줍니다. 소문자로 바꾸려면 새 문자열이 필요하므로
빌려서 돌려줄 수 없습니다.

## 연습문제

### 문제 1 [읽기]

아래 함수 서명 중 관례에 맞는 것은 무엇입니까?

```rust
// (가)
fn count_lines(text: String) -> usize

// (나)
fn count_lines(text: &str) -> usize

// (다)
fn build_path(repo: &str, file: &str) -> String

// (라)
fn build_path(repo: String, file: String) -> String
```

<details>
<summary>정답 보기</summary>

(나)와 (다)가 관례에 맞습니다.

(가)는 줄 수만 세면서 소유권을 가져갑니다. 호출한 쪽에서 그 문자열을 더 쓸
수 없게 되므로 불필요한 제약입니다.

(라)는 두 조각을 받아 합치는 함수인데 소유권을 가져갑니다. 호출하는 쪽에서
`.to_string()`을 불러야 하고 그것은 낭비입니다.

규칙은 간단합니다. **값을 보관하지 않으면 `&str`로 받습니다.**

</details>

### 문제 2 [고치기]

```bash
cd book/exercises
cargo test -p ex_01_05_a
```

`String`과 `&str`을 잘못 쓴 코드를 고치는 문제입니다.

### 문제 3 [고치기]

```bash
cargo test -p ex_01_05_b
```

함수 서명을 `&str`로 바꿔 불필요한 복사를 없애는 문제입니다.

### 문제 4 [쓰기]

```bash
cargo test -p ex_01_05_c
```

`format!`으로 nunchi의 `NodeId` 형식을 만드는 문제입니다.

## 정리

`String`은 소유하고 `&str`은 빌립니다. 코드에 직접 쓴 문자열은 `&str`입니다.

함수 매개변수에는 `&str`, 구조체 필드에는 `String`, 함수 반환값에는 대개
`String`을 씁니다. `format!`은 읽기만 하므로 그 뒤에 원본의 소유권을 넘길 수
있습니다.

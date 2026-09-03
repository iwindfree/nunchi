# 0.3 타입 표기 읽는 법

> **선행 장**: [0.2 변수와 `let`](00-2-variables.md)
> **연습문제**: 2개

이 장의 목표는 타입을 **읽는 것**입니다. 타입을 직접 설계하는 방법은 나중에
다룹니다. 지금은 코드에 적힌 타입이 무슨 뜻인지 알아보는 것만으로 충분합니다.

## 설명

### 타입을 적는 자리

Rust는 대부분의 경우 타입을 추론하므로 적지 않아도 됩니다.

```rust
let count = 3;              // 컴파일러가 i32 로 추론합니다
let count: u32 = 3;         // 직접 적을 수도 있습니다
```

**함수는 예외입니다.** 매개변수와 반환 타입은 반드시 적어야 합니다.

```rust
fn estimate_tokens(text: &str) -> usize {
    //            ^^^^^^^^^^^     ^^^^^
    //            매개변수 타입    반환 타입
}
```

이것은 제약이 아니라 도움입니다. 함수 서명만 보면 무엇을 받아 무엇을 주는지
알 수 있고, 함수 안을 읽지 않아도 됩니다.

### 기본 타입

| 표기 | 뜻 | 예 |
|---|---|---|
| `i32`, `i64` | 부호 있는 정수 | `-5` |
| `u32`, `u64`, `usize` | 부호 없는 정수 | `5` |
| `f32`, `f64` | 실수 | `0.5` |
| `bool` | 참과 거짓 | `true` |
| `char` | 문자 하나 | `'a'` |
| `&str` | 문자열 조각 | `"hello"` |
| `String` | 소유한 문자열 | 1.5장에서 다룹니다 |

`usize`는 크기와 개수에 쓰는 정수입니다. 컴퓨터에 따라 크기가 달라지며, 64비트
환경에서는 `u64`와 같습니다. 배열의 길이나 인덱스에는 `usize`를 씁니다.

### 각괄호 안의 타입

여기가 처음 보면 낯선 부분입니다. 타입 뒤에 각괄호가 붙는 경우가 있습니다.

```rust
Vec<String>
Option<usize>
HashMap<String, Vec<NodeId>>
```

읽는 방법은 간단합니다. **각괄호 안은 "무엇을 담는가"입니다.**

| 표기 | 읽는 법 |
|---|---|
| `Vec<String>` | 문자열을 담는 목록 |
| `Option<usize>` | 있을 수도 없을 수도 있는 정수 |
| `Result<Node, Error>` | 성공하면 `Node`, 실패하면 `Error` |
| `HashMap<String, usize>` | 문자열을 키로, 정수를 값으로 갖는 표 |
| `Vec<(Span, NodeId)>` | `Span`과 `NodeId`가 짝지어진 목록 |

각괄호가 겹칠 수도 있습니다.

```rust
HashMap<String, Vec<NodeId>>
```

안쪽부터 읽습니다. `Vec<NodeId>`는 `NodeId` 목록이고, 전체는 "문자열을 키로,
`NodeId` 목록을 값으로 갖는 표"입니다. nunchi에서 심볼 이름 하나에 여러 정의가
대응될 수 있으므로 이런 타입이 쓰입니다.

### 앞에 붙는 `&`

타입 앞에 `&`가 붙으면 "빌린 것"입니다.

```rust
fn estimate_tokens(text: &str) -> usize
//                       ^ 빌렸습니다
```

무엇을 빌린다는 뜻인지는 1부에서 자세히 다룹니다. 지금은 **`&`가 있으면 원본을
가져오지 않고 보기만 한다**고 이해하시면 충분합니다.

## 이 프로젝트에서는

실제 코드에서 타입을 하나 읽어 보겠습니다.

```rust
// crates/nunchi-core/src/resolve.rs 에서
pub fn resolve_call(
    &self,
    callee: &str,
    stats: &mut ResolveStats,
    tally: &mut UnresolvedTally,
) -> Vec<(NodeId, f32)> {
```

한 줄씩 읽습니다.

- `&self`는 이 함수가 속한 값을 빌려서 봅니다. 5.1장에서 다룹니다.
- `callee: &str`는 문자열을 빌려서 받습니다.
- `stats: &mut ResolveStats`는 `ResolveStats`를 빌리면서 **바꿀 수 있게** 받습니다.
  `mut`이 붙었기 때문입니다.
- `-> Vec<(NodeId, f32)>`는 `NodeId`와 실수가 짝지어진 목록을 돌려줍니다.

이 서명만 보고도 함수가 무엇을 하는지 짐작할 수 있습니다. 호출 대상 이름을
받아서, 통계를 갱신하면서, 후보 목록과 각 후보의 점수를 돌려줍니다.

## 연습문제

### 문제 1 [읽기]

아래 타입들을 한국어로 읽어 보십시오.

```rust
Vec<Edge>
Option<String>
HashMap<String, Vec<(Span, NodeId)>>
&[String]
```

<details>
<summary>정답 보기</summary>

- `Vec<Edge>`는 엣지를 담는 목록입니다.
- `Option<String>`은 있을 수도 없을 수도 있는 문자열입니다.
- `HashMap<String, Vec<(Span, NodeId)>>`는 문자열을 키로 갖고,
  값으로는 `Span`과 `NodeId`가 짝지어진 목록을 갖는 표입니다.
- `&[String]`은 빌린 문자열 목록입니다. `Vec`이 아니라 슬라이스이며
  6.1장에서 다룹니다. 지금은 "목록을 빌린 것"으로 이해하시면 됩니다.

</details>

### 문제 2 [고치기]

```bash
cd book/exercises
cargo test -p ex_00_03_a
```

함수 서명에서 타입이 맞지 않는 문제입니다.

## 정리

타입은 변수에서 생략할 수 있지만 함수 서명에는 반드시 적습니다. 각괄호 안은
"무엇을 담는가"이며 안쪽부터 읽습니다. 앞에 `&`가 붙으면 빌린 것입니다.

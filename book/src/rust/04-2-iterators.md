# 4.2 이터레이터

> **선행 장**: [4.1 클로저](04-1-closures.md), [1.3 빌림](01-3-borrow.md)
> **연습문제**: 4개

이터레이터는 값을 하나씩 꺼내 주는 장치입니다. nunchi 코드에서 `.iter()`가
134번 나오고, 이터레이터 메서드를 합치면 581번입니다. **이 코드의 중심 구조입니다.**

## 반복문 대신 씁니다

같은 일을 두 방식으로 씁니다.

```rust
// 반복문
let mut total = 0;
for c in counts {
    total += c;
}

// 이터레이터
let total: u32 = counts.iter().sum();
```

이터레이터 쪽이 짧지만, 짧다는 것만이 이유는 아닙니다.

**첫째, `mut` 변수가 사라집니다.** 위 반복문은 `total`을 계속 바꿉니다.
이터레이터 방식에는 바뀌는 값이 없으므로 중간 상태를 추적할 필요가 없습니다.

**둘째, 무엇을 하는지가 드러납니다.** `sum()`이라고 적혀 있으면 합을 구한다는
사실이 바로 보입니다. 반복문은 안을 읽어야 압니다.

**셋째, 실수할 자리가 줄어듭니다.** 인덱스를 잘못 다루거나 초기값을 잊는 실수가
생길 수 없습니다.

## 세 가지 `iter`

여기가 소유권과 만나는 지점이며 처음에 헷갈립니다.

| 메서드 | 꺼내 주는 것 | 원본은 |
|---|---|---|
| `.iter()` | `&T` (빌린 것) | 그대로 쓸 수 있습니다 |
| `.iter_mut()` | `&mut T` | 바꿀 수 있습니다 |
| `.into_iter()` | `T` (소유권) | 사라집니다 |

```rust
let names = vec!["a".to_string(), "b".to_string()];

for n in names.iter() { }        // n 은 &String 입니다
println!("{:?}", names);          // 원본을 계속 씁니다

for n in names.into_iter() { }   // n 은 String 입니다
println!("{:?}", names);          // 오류입니다. 소유권이 넘어갔습니다
```

`for n in names`처럼 그냥 쓰면 `.into_iter()`가 호출됩니다. 그래서 반복 뒤에
원본을 쓸 수 없게 됩니다.

```rust
for n in &names { }              // .iter() 와 같습니다
println!("{:?}", names);          // 씁니다
```

**대부분의 경우 `.iter()`나 `&`를 씁니다.** 소유권이 필요할 때만
`.into_iter()`를 씁니다.

## 세 단계로 나뉩니다

이터레이터를 쓰는 코드는 언제나 세 부분입니다.

```
만든다  →  변형한다  →  거둔다
.iter()    .map()      .collect()
           .filter()   .sum()
                       .count()
```

**변형 단계는 아무것도 실행하지 않습니다.** `.map()`을 불러도 그 자리에서는
계산이 일어나지 않고, 마지막에 소비하는 단계에서야 한꺼번에 실행됩니다.

```rust
let lengths = names.iter().map(|n| n.len());     // 아직 아무 일도 안 했습니다
let total: usize = lengths.sum();                // 여기서 실행됩니다
```

이것을 지연 평가(lazy evaluation)라고 부릅니다. 덕분에 여러 단계를 이어도
목록을 여러 번 훑지 않습니다. 한 번만 훑으면서 모든 단계를 적용합니다.

## 자주 쓰는 변형

| 메서드 | 하는 일 |
|---|---|
| `.map(f)` | 각 값에 `f`를 적용합니다 |
| `.filter(pred)` | 조건에 맞는 것만 남깁니다 |
| `.filter_map(f)` | `f`가 `Some`을 준 것만 남기고 값을 꺼냅니다 |
| `.take(n)` | 앞에서 `n`개만 봅니다 |
| `.skip(n)` | 앞에서 `n`개를 건너뜁니다 |
| `.enumerate()` | 번호를 붙여 `(순번, 값)`으로 만듭니다 |
| `.chain(other)` | 다른 이터레이터를 이어 붙입니다 |
| `.flatten()` | 중첩을 한 단계 풀어 줍니다 |
| `.rev()` | 뒤에서부터 봅니다 |

## 자주 쓰는 소비 메서드

| 메서드 | 하는 일 |
|---|---|
| `.collect()` | `Vec`이나 `HashMap` 등으로 모읍니다 |
| `.count()` | 개수를 셉니다 |
| `.sum()`, `.max()`, `.min()` | 합, 최댓값, 최솟값을 구합니다 |
| `.any(pred)` | 하나라도 조건에 맞으면 참입니다 |
| `.all(pred)` | 모두 조건에 맞으면 참입니다 |
| `.find(pred)` | 조건에 맞는 첫 값을 찾습니다 |
| `.find_map(f)` | `f`가 처음으로 `Some`을 준 값을 돌려줍니다 |
| `.position(pred)` | 조건에 맞는 첫 값의 순번을 찾습니다 |

`.any()`와 `.all()`은 답이 정해지면 바로 멈춥니다. 첫 값에서 참이 나오면
나머지는 보지 않습니다.

## 이 프로젝트에서는

### 거르고 모으기

```rust
// crates/nunchi-core/src/semantic.rs 에서
pub fn split_identifier(ident: &str) -> Vec<String> {
    // ... 앞부분 생략
    parts.retain(|p| p.len() > 1);
    parts
}
```

`retain`은 조건에 맞는 것만 남기고 나머지를 지웁니다. 한 글자짜리 조각은
검색에 도움이 안 되므로 버립니다.

### 조건 확인

```rust
// crates/nunchi-core/src/framework.rs 에서
fn has_function_argument(args: Node) -> bool {
    let mut cursor = args.walk();
    let found = args.children(&mut cursor).any(|a| {
        matches!(
            a.kind(),
            "arrow_function" | "function_expression" | "function" | "function_declaration"
        )
    });
    found
}
```

인자 중에 함수가 하나라도 있으면 참입니다. `this.post('/users', handler)`처럼
핸들러를 넘기는 호출은 클라이언트 호출이 아니라 라우트 등록이므로, 이
판정으로 걸러 냅니다.

`.any()`가 함수를 찾는 순간 멈추므로 나머지 인자는 보지 않습니다.

### 첫 번째를 찾기

```rust
// crates/nunchi-core/src/rules.rs 에서
pub fn route_for(&self, lang: &str, annotation: &str) -> Option<&RouteRule> {
    self.route
        .iter()
        .find(|r| Self::lang_matches(&r.lang, lang) && r.annotation == annotation)
}
```

규칙 목록에서 조건에 맞는 첫 규칙을 찾습니다. 없으면 `None`입니다.

`.iter()`를 썼으므로 `find`가 `Option<&RouteRule>`을 돌려줍니다. 규칙을
복사하지 않고 빌려서 돌려줍니다.

### 번호 붙이기

```rust
// crates/nunchi-core/src/pack.rs 에서
for (rank_pos, (idx, score, why)) in scored.iter().enumerate() {
```

`enumerate()`가 `(순번, 값)`을 만듭니다. 값 자체도 튜플이므로 두 겹으로
풀었습니다. `rank_pos`는 순위이고 나머지 셋이 원래 값입니다.

순위가 필요한 이유는 상위 두세 개만 전체 본문을 담기 때문입니다.

## 연습문제

### 문제 1 [읽기]

아래 코드가 컴파일되지 않는 이유는 무엇입니까?

```rust
let names = vec!["a".to_string(), "b".to_string()];
let total: usize = names.into_iter().map(|n| n.len()).sum();
println!("{:?}", names);
```

<details>
<summary>정답 보기</summary>

`.into_iter()`가 `names`의 소유권을 가져갔기 때문입니다.

`.iter()`로 바꾸면 됩니다.

```rust
let total: usize = names.iter().map(|n| n.len()).sum();
println!("{:?}", names);        // 이제 됩니다
```

`.iter()`는 `&String`을 꺼내 주고, `.len()`은 빌린 것으로도 부를 수
있습니다. 원본은 그대로 남습니다.

</details>

### 문제 2 [고치기]

```bash
cd book/exercises
cargo test -p ex_04_02_a
```

`.into_iter()`를 `.iter()`로 바꾸는 문제입니다.

### 문제 3 [쓰기]

```bash
cargo test -p ex_04_02_b
```

`.any()`와 `.find()`를 쓰는 문제입니다.

### 문제 4 [고치기]

```bash
cargo test -p ex_04_02_c
```

반복문을 이터레이터로 바꾸는 문제입니다.

## 정리

이터레이터는 값을 하나씩 꺼내 주며, 만들고 변형하고 소비하는 세 단계를
거칩니다. 변형 단계는 실행되지 않고 소비하는 단계에서 한꺼번에 실행됩니다.

`.iter()`는 빌리고 `.into_iter()`는 소유권을 가져갑니다. `for x in v`는
후자이므로 원본을 쓸 수 없게 됩니다. 대부분 `.iter()`나 `&`를 씁니다.

`.any()`와 `.find()`는 답이 정해지면 바로 멈춥니다.

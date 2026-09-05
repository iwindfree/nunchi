# 1.3 빌림 `&`와 `&mut`

> **선행 장**: [1.2 이동과 복사](01-2-move.md)
> **연습문제**: 4개

앞 장에서 소유권을 넘기지 않고 값을 쓰는 방법이 필요하다고 했습니다. 그 방법이
빌림입니다. nunchi 코드에서 가장 흔하게 보게 될 문법입니다.

## 설명

`&`를 붙이면 소유권을 넘기지 않고 값을 볼 수 있습니다.

```rust
let name = String::from("OrderService");
let length = measure(&name);      // 빌려서 넘깁니다
println!("{}", name);             // name 은 여전히 소유자입니다

fn measure(s: &String) -> usize {
    s.len()
}
```

`&name`을 참조(reference)라고 부릅니다. 원본을 가리키는 화살표라고 생각하시면
됩니다.

```
   name (소유자)                    힙
┌──────────────┐              ┌─────────────────┐
│ 주소 ────────┼─────────────▶│ "OrderService"  │
└──────────────┘              └─────────────────┘
        ▲
        │ &name (빌린 것)
┌───────┴──────┐
│ s            │   함수 안에서 보고만 있습니다
└──────────────┘
```

빌린 쪽은 값을 **볼 수만** 있습니다. 바꾸려면 `&mut`이 필요합니다.

```rust
fn add_suffix(s: &mut String) {
    s.push_str("Impl");
}

let mut name = String::from("OrderService");
add_suffix(&mut name);
println!("{}", name);      // "OrderServiceImpl"
```

`&mut`를 쓰려면 원본도 `mut`이어야 합니다. 바꿀 수 없는 변수를 빌려서 바꿀
수는 없기 때문입니다.

## 규칙 두 가지

빌림에는 규칙이 있습니다. 이 규칙이 Rust에서 가장 중요합니다.

> **규칙 1**: `&`(읽기 전용)는 동시에 여러 개 가능합니다.
> **규칙 2**: `&mut`(변경 가능)는 동시에 하나만 가능하며, 그때는 `&`도 있을 수 없습니다.

한 문장으로 줄이면 이렇습니다. **읽는 사람은 여럿이어도 되지만, 쓰는 사람이
있으면 그 사람 혼자여야 합니다.**

```rust
let mut data = vec![1, 2, 3];

let a = &data;
let b = &data;              // 읽기 전용은 여러 개 가능합니다
println!("{:?} {:?}", a, b);

let c = &mut data;          // 변경 가능은 하나만 가능합니다
c.push(4);
```

아래는 오류가 납니다.

```rust
let mut data = vec![1, 2, 3];
let a = &data;              // 읽고 있습니다
let b = &mut data;          // 동시에 바꾸려고 합니다
println!("{:?}", a);        // 오류입니다
```

### 왜 이 규칙이 필요한가

읽는 중에 다른 곳에서 바꾸면 문제가 생깁니다.

```rust
let mut items = vec![1, 2, 3];
let first = &items[0];       // 첫 원소를 가리킵니다
items.push(4);               // 목록이 커지면서 힙 위치가 바뀔 수 있습니다
println!("{}", first);       // 옛 주소를 읽게 됩니다
```

`Vec`은 용량이 부족하면 더 큰 공간을 새로 잡고 데이터를 옮깁니다. 그러면
`first`가 가리키던 주소는 이미 해제된 메모리가 됩니다. C++에서 이것이
실제로 자주 발생하던 오류입니다.

Rust는 이 코드를 컴파일하지 않습니다. `first`가 살아 있는 동안 `items`를
바꿀 수 없기 때문입니다.

### 빌림이 언제까지 살아 있는가

빌린 것은 **마지막으로 쓰이는 지점까지** 살아 있습니다. 선언한 블록 끝까지가
아닙니다.

```rust
let mut data = vec![1, 2, 3];

let a = &data;
println!("{:?}", a);        // a 를 마지막으로 씁니다. 여기서 빌림이 끝납니다

let b = &mut data;          // 그래서 이제 됩니다
b.push(4);
```

이 동작 덕분에 실제로는 규칙이 그렇게 답답하지 않습니다. 컴파일러가 필요한
범위만 정확히 계산합니다.

## 이 프로젝트에서는

nunchi 함수 서명 대부분이 빌림을 씁니다. 세 가지 형태를 보겠습니다.

### 읽기만 하는 함수

```rust
// crates/nunchi-core/src/framework.rs 에서
pub fn tables_in_sql(sql: &str) -> Vec<(String, String)> {
```

SQL 문자열을 읽어서 테이블 이름을 뽑아냅니다. 원본을 보관하지 않으므로 빌리기만
합니다. **값을 보관하지 않는 함수는 빌려 받는다**가 일반 관례입니다.

### 바꾸는 함수

```rust
// crates/nunchi-core/src/store/sqlite.rs 에서
pub fn upsert_nodes(&mut self, nodes: &[Node]) -> Result<usize> {
```

두 가지가 섞여 있습니다.

- `&mut self`는 저장소 자신을 바꿉니다. 데이터베이스에 쓰기 때문입니다.
- `nodes: &[Node]`는 노드 목록을 읽기만 합니다.

서명만 보고 "이 함수는 저장소를 바꾸지만 넘긴 노드 목록은 그대로 둔다"는 사실을
알 수 있습니다.

### 여러 개를 동시에 빌려야 할 때

```rust
// crates/nunchi-core/src/index.rs 에서
fn scan_repo(
    repo: &str,
    root: &Path,
    config: &Config,
    excludes: &GlobSet,
    rules: &crate::rules::FrameworkRules,
    store: &mut SqliteStore,
    stats: &mut IndexStats,
    table: &mut SymbolTable,
    // ...
) -> Result<Vec<String>> {
```

매개변수가 많습니다. 읽기만 하는 것은 `&`, 바꾸는 것은 `&mut`입니다.

이 함수가 매개변수를 이렇게 많이 받는 이유가 규칙 2와 관련이 있습니다.
`store`, `stats`, `table`을 **서로 다른 값**으로 빌리므로 규칙 2에
걸리지 않습니다. 만약 이 셋을 하나의 구조체에 담고 그 구조체를 `&mut`로
빌렸다면, 그 안의 세 필드를 동시에 다른 함수에 넘기기 어려워집니다.

이것이 Rust 코드에서 구조체를 잘게 나누는 이유입니다.

## 연습문제

### 문제 1 [읽기]

아래 코드가 컴파일되는지 판단하십시오.

```rust
// (가)
let mut v = vec![1, 2, 3];
let a = &v;
let b = &v;
println!("{:?} {:?}", a, b);

// (나)
let mut v = vec![1, 2, 3];
let a = &mut v;
let b = &mut v;
a.push(4);

// (다)
let mut v = vec![1, 2, 3];
let a = &v;
println!("{:?}", a);
let b = &mut v;
b.push(4);
```

<details>
<summary>정답 보기</summary>

(가)와 (다)가 컴파일됩니다.

(가)는 읽기 전용을 두 개 빌렸으므로 규칙 1에 맞습니다.

(나)는 변경 가능을 두 개 빌렸으므로 규칙 2에 걸립니다.

(다)는 `a`를 `println!`에서 마지막으로 쓰고 나면 빌림이 끝납니다. 그래서
그 뒤에 `&mut`를 빌릴 수 있습니다.

</details>

### 문제 2 [고치기]

```bash
cd book/exercises
cargo test -p ex_01_03_a
```

읽기와 쓰기가 겹치는 코드를 고치는 문제입니다.

### 문제 3 [고치기]

```bash
cargo test -p ex_01_03_b
```

함수 서명을 `&`로 바꾸는 문제입니다.

### 문제 4 [쓰기]

```bash
cargo test -p ex_01_03_c
```

`&mut`로 값을 바꾸는 함수를 작성하는 문제입니다.

## 정리

`&`는 읽기만 하는 빌림이고 여러 개 가능합니다. `&mut`는 바꿀 수 있는 빌림이며
하나만 가능하고 그때는 `&`도 있을 수 없습니다.

빌린 것은 마지막으로 쓰이는 지점까지만 살아 있습니다.

값을 보관하지 않는 함수는 빌려 받는 것이 관례입니다. nunchi 함수 대부분이
그렇게 되어 있습니다.

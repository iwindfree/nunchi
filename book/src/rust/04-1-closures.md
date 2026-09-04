# 4.1 클로저

> **선행 장**: [1.2 이동과 복사](01-2-move.md), [0.2 변수와 `let`](00-2-variables.md)
> **연습문제**: 4개

클로저는 이름 없는 함수입니다. nunchi 코드에 262번 나오며, 다음 장에서 다룰
이터레이터와 짝을 이룹니다.

## 문법

세로 막대 사이에 매개변수를 적습니다.

```rust
let double = |x| x * 2;
println!("{}", double(3));      // 6
```

`fn`으로 만든 함수와 두 가지가 다릅니다.

**첫째, 타입을 적지 않아도 됩니다.** 쓰이는 자리에서 컴파일러가 추론합니다.

```rust
fn double(x: i32) -> i32 { x * 2 }      // 함수는 적어야 합니다
let double = |x| x * 2;                  // 클로저는 안 적어도 됩니다
```

**둘째, 바깥 변수를 쓸 수 있습니다.** 이것이 클로저의 핵심입니다.

```rust
let prefix = "file:";
let make_id = |path| format!("{prefix}{path}");     // prefix 를 씁니다
```

함수는 이렇게 할 수 없습니다. 함수 안에서는 매개변수와 자기 안에서 만든
변수만 쓸 수 있습니다.

## 바깥 변수를 어떻게 가져가는가

여기가 소유권과 만나는 지점입니다. 클로저가 바깥 변수를 쓸 때, 세 가지 방식
중 하나로 가져갑니다.

| 방식 | 언제 | 결과 |
|---|---|---|
| 빌림 `&` | 읽기만 할 때 | 바깥에서도 계속 쓸 수 있습니다 |
| 변경 빌림 `&mut` | 바꿀 때 | 클로저를 쓰는 동안 바깥에서 못 씁니다 |
| 이동 | 소유권이 필요할 때 | 바깥에서 못 씁니다 |

**컴파일러가 알아서 고릅니다.** 클로저 안에서 무엇을 하는지 보고 가장 약한
방식을 선택합니다.

```rust
let names = vec!["a".to_string()];

let count = || names.len();          // 읽기만 하므로 빌립니다
println!("{}", count());
println!("{:?}", names);             // 바깥에서도 씁니다. 문제없습니다
```

## `move` 키워드

`move`를 붙이면 **반드시 소유권을 가져갑니다.**

```rust
let names = vec!["a".to_string()];
let count = move || names.len();     // 소유권을 가져갑니다
println!("{:?}", names);             // 오류입니다. 이미 넘어갔습니다
```

왜 이런 것이 필요합니까? **클로저가 만들어진 곳보다 오래 살아야 할 때**입니다.

빌려서 가져가면 원본이 사라지는 순간 클로저도 쓸 수 없게 됩니다. 클로저를
다른 스레드에 넘기거나 구조체에 저장하려면 소유권이 필요합니다.

## 이 프로젝트에서는

### 짧은 클로저

가장 흔한 형태입니다. 값 하나를 바꿔서 돌려줍니다.

```rust
// crates/nunchi-core/src/index.rs 에서
.map(|d| d.as_secs() as i64)
```

```rust
// crates/nunchi-core/src/semantic.rs 에서
.filter(|p| p.len() > 1)
```

이런 클로저는 다음 장의 이터레이터와 함께 쓰입니다.

### 값을 만드는 클로저

`unwrap_or_else`와 `with_context`가 클로저를 받습니다. 필요할 때만 실행하기
위해서입니다.

```rust
// crates/nunchi-core/src/config.rs 에서
.with_context(|| format!("설정 파일을 읽을 수 없습니다: {}", path.display()))?
```

매개변수가 없으므로 `||`가 비어 있습니다. 오류가 났을 때만 이 메시지를
만듭니다.

### `move`가 필요한 곳

nunchi에서 `move`를 쓰는 자리가 파일 워크입니다.

```rust
// crates/nunchi-core/src/index.rs 에서
let prune_root = root.to_path_buf();
let prune_set = excludes.clone();
let walker = ignore::WalkBuilder::new(root)
    .hidden(true)
    .git_ignore(true)
    .filter_entry(move |entry| {
        let Some(rel) = npath::relative_to(&prune_root, entry.path()) else {
            return true;
        };
        if rel.is_empty() {
            return true;
        }
        if entry.file_type().is_some_and(|t| t.is_dir()) {
            !(prune_set.is_match(&rel) || prune_set.is_match(format!("{rel}/")))
        } else {
            !prune_set.is_match(&rel)
        }
    })
    .build();
```

[1.4장](01-4-clone.md)에서 미뤄 둔 설명을 여기서 합니다.

`filter_entry`에 넘긴 클로저는 **`walker` 안에 저장됩니다.** 그리고 아래
`for entry in walker` 반복이 진행되는 내내 살아 있어야 합니다.

빌려서 가져갔다면 컴파일러가 거부합니다. 클로저가 `walker` 안에 들어가는데,
빌린 대상인 `root`와 `excludes`가 언제까지 살아 있는지 보장할 수 없기
때문입니다.

그래서 `move`를 붙였고, `move`를 붙였으므로 소유권을 넘길 값이 필요했습니다.
`root`는 `&Path`라서 넘길 수 없으므로 `to_path_buf()`로 소유한 값을 만들었고,
`excludes`는 함수의 다른 곳에서도 쓰이므로 `clone()`으로 복사했습니다.

**이것이 그 `clone`의 이유입니다.** 저장소마다 한 번씩만 일어나므로 실제
비용은 무시할 수준이라 고치지 않았습니다.

### 워처의 클로저

```rust
// crates/nunchi-cli/src/watch.rs 에서
let (tx, rx) = mpsc::channel::<Event>();
let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
    if let Ok(event) = res {
        let _ = tx.send(event);
    }
})?;
```

파일 변경이 생길 때마다 이 클로저가 호출됩니다. **다른 스레드에서 호출됩니다.**
그래서 `tx`의 소유권을 가져가야 하고, `move`가 필수입니다.

## 클로저를 받는 함수를 읽는 법

라이브러리 문서에서 이런 서명을 보게 됩니다.

```rust
fn map<B, F>(self, f: F) -> Map<Self, F>
where
    F: FnMut(Self::Item) -> B
```

`F: FnMut(...)` 부분이 "클로저를 받는다"는 뜻입니다. 세 종류가 있습니다.

| 표기 | 뜻 |
|---|---|
| `Fn` | 빌려서 읽기만 합니다. 여러 번 부를 수 있습니다 |
| `FnMut` | 바깥 변수를 바꿉니다. 여러 번 부를 수 있습니다 |
| `FnOnce` | 소유권을 가져갑니다. 한 번만 부를 수 있습니다 |

**직접 쓸 일은 거의 없습니다.** 클로저를 넘길 때는 컴파일러가 알아서
맞춰 줍니다. 문서를 읽을 때 이 표기가 무슨 뜻인지만 알면 충분합니다.

nunchi 코드에 이 표기가 나오지 않는 이유도 그것입니다. 클로저를 받는 함수를
직접 만들지 않고 라이브러리 것만 쓰기 때문입니다.

## 연습문제

### 문제 1 [읽기]

아래 코드가 컴파일되는지 판단하십시오.

```rust
// (가)
let names = vec!["a".to_string()];
let count = || names.len();
println!("{}", count());
println!("{:?}", names);

// (나)
let names = vec!["a".to_string()];
let count = move || names.len();
println!("{}", count());
println!("{:?}", names);
```

<details>
<summary>정답 보기</summary>

(가)는 컴파일되고 (나)는 되지 않습니다.

(가)의 클로저는 `names.len()`으로 읽기만 하므로 컴파일러가 빌림을
선택합니다. 바깥에서도 계속 쓸 수 있습니다.

(나)는 `move`를 붙였으므로 소유권을 가져갑니다. 그래서 마지막 줄에서
`names`를 쓸 수 없습니다.

`move`는 필요할 때만 붙입니다. 클로저가 만들어진 곳보다 오래 살아야 할 때가
그런 경우입니다.

</details>

### 문제 2 [고치기]

```bash
cd book/exercises
cargo test -p ex_04_01_a
```

`move`가 필요한 곳에 붙이는 문제입니다.

### 문제 3 [고치기]

```bash
cargo test -p ex_04_01_b
```

불필요한 `move` 때문에 바깥 변수를 못 쓰게 된 코드를 고치는 문제입니다.

### 문제 4 [쓰기]

```bash
cargo test -p ex_04_01_c
```

`unwrap_or_else`에 클로저를 넘기는 문제입니다.

## 정리

클로저는 이름 없는 함수이며 바깥 변수를 쓸 수 있습니다. 가져가는 방식은
컴파일러가 자동으로 고르며, 읽기만 하면 빌리고 바꾸면 변경 빌림을 씁니다.

`move`를 붙이면 반드시 소유권을 가져갑니다. 클로저가 만들어진 곳보다 오래
살아야 할 때 필요합니다. 다른 스레드로 넘기거나 구조체에 저장할 때가
그렇습니다.

`Fn`, `FnMut`, `FnOnce`는 클로저를 받는 함수의 서명에 나오며, 직접 쓸 일은
거의 없습니다.

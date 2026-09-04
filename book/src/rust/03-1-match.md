# 3.1 `match`

> **선행 장**: [0.4 구조체, 열거형, 튜플](00-4-data.md), [2.1 `Option<T>`](02-1-option.md)
> **연습문제**: 3개

`match`는 값이 어떤 경우인지 판별하여 분기하는 문법입니다. nunchi 코드에 57번
나옵니다.

## 다른 언어의 `switch`와 무엇이 다른가

`switch`와 비슷해 보이지만 세 가지가 다릅니다.

**첫째, 모든 경우를 다뤄야 합니다.** 빠뜨리면 컴파일되지 않습니다.

```rust
enum Provenance { Fast, Precise }

fn label(p: Provenance) -> &'static str {
    match p {
        Provenance::Fast => "빠른 경로",
        // Precise 를 빠뜨리면 컴파일 오류입니다
    }
}
```

이것을 완전성 검사(exhaustiveness check)라고 부릅니다. **열거형에 값을
추가하면 그것을 다루지 않은 모든 `match`가 컴파일 오류가 됩니다.** 고쳐야 할
곳을 컴파일러가 전부 찾아 줍니다.

nunchi에서 실제로 효과가 있었습니다. 엣지 종류를 19개까지 늘리는 동안, 새 종류를
추가할 때마다 어디를 고쳐야 하는지 컴파일러가 알려 줬습니다.

**둘째, 값을 꺼낼 수 있습니다.** 열거형이 품고 있는 값을 그 자리에서 꺼냅니다.

```rust
enum Verified {
    Fresh(String),
    Unknown,
    Stale,
}

match source {
    Verified::Fresh(text) => text.lines().count(),   // text 를 꺼내 씁니다
    Verified::Unknown => 0,
    Verified::Stale => 0,
}
```

`Fresh`인 경우에만 `text`가 있고, 나머지 경우에는 없습니다. **없는 경우에
`text`를 쓰려고 하면 컴파일되지 않습니다.**

**셋째, 값을 돌려줍니다.** `match`는 문장이 아니라 식이므로 결과를 변수에
넣을 수 있습니다.

```rust
let count = match source {
    Verified::Fresh(text) => text.lines().count(),
    _ => 0,
};
```

## `_` 는 나머지 전부입니다

모든 경우를 일일이 적기 번거로우면 `_`로 묶습니다.

```rust
match ext {
    "rs" => "rust",
    "java" => "java",
    _ => return None,        // 나머지 전부입니다
}
```

문자열이나 숫자처럼 경우가 무한한 값에는 `_`가 반드시 필요합니다.

다만 **열거형에는 `_`를 되도록 쓰지 않는 편이 낫습니다.** `_`를 쓰면 나중에
값을 추가해도 컴파일러가 알려 주지 않기 때문입니다. 완전성 검사의 이득이
사라집니다.

## 여러 값을 한 갈래로 묶습니다

```rust
match kind {
    NodeKind::Symbol | NodeKind::File | NodeKind::Route => true,
    _ => false,
}
```

`|`로 여러 경우를 묶습니다.

## 조건을 덧붙입니다

`if`를 붙여 조건을 더할 수 있습니다. 이것을 가드(guard)라고 부릅니다.

```rust
// crates/nunchi-core/src/resolve.rs 에서
match candidates.len() {
    0 => {
        stats.unresolved += 1;
        tally.record(callee);
        Vec::new()
    }
    1 => {
        stats.resolved += 1;
        vec![(candidates[0].clone(), 0.8)]
    }
    n if n <= MAX_CANDIDATES => {        // 가드입니다
        stats.ambiguous += 1;
        let confidence = 0.8 / n as f32;
        candidates.iter().cloned().map(|id| (id, confidence)).collect()
    }
    _ => {
        stats.dropped += 1;
        tally.record(callee);
        Vec::new()
    }
}
```

이름 해소 결과를 후보 개수에 따라 네 갈래로 나눕니다.

- 후보가 없으면 외부 라이브러리 호출로 봅니다.
- 하나면 신뢰할 수 있습니다.
- 둘이나 셋이면 모두 연결하되 신뢰도를 나눕니다.
- 그보다 많으면 포기합니다. `get`이나 `build` 같은 흔한 이름이 그래프를
  오염시키기 때문입니다.

`n if n <= MAX_CANDIDATES`에서 `n`은 `candidates.len()`의 값을 받은
이름입니다. 그 값에 조건을 걸었습니다.

## 튜플을 한꺼번에 따집니다

```rust
// crates/nunchi-core/src/pack.rs 에서
let (doc, body) = match tier {
    Tier::L0 => (None, None),
    Tier::L1 => { /* ... */ }
    Tier::L2 => { /* ... */ }
};
```

여러 값을 동시에 따질 수도 있습니다.

```rust
match (start, end) {
    (Some(s), Some(e)) => Some(Span { start_line: s, end_line: e }),
    _ => None,
}
```

둘 다 있을 때만 `Span`을 만들고, 하나라도 없으면 `None`입니다.

## 이 프로젝트에서는

`match`가 가장 잘 쓰인 곳을 봅니다.

```rust
// crates/nunchi-core/src/framework.rs 에서
let (method, url_arg) = clients.iter().find_map(|rule| -> Option<(String, usize)> {
    match func.kind() {
        "identifier" => {
            let name = text(func, src);
            (rule.callee.as_deref() == Some(name))
                .then(|| (rule.method.clone().unwrap_or_else(|| "GET".into()), rule.url_arg))
        }
        "member_expression" => {
            // axios.get(...) 형태입니다
            let prop = func.child_by_field_name("property")?;
            // ...
        }
        _ => None,
    }
})?;
```

tree-sitter가 알려 준 노드 종류에 따라 분기합니다. `fetch("/api/x")`와
`axios.get("/api/x")`는 트리 모양이 다르므로 각각 다르게 처리해야 합니다.

## 연습문제

### 문제 1 [읽기]

아래 코드가 컴파일되지 않는 이유는 무엇입니까?

```rust
enum EdgeKind { Calls, Imports, Injects }

fn is_structural(k: EdgeKind) -> bool {
    match k {
        EdgeKind::Calls => true,
        EdgeKind::Imports => true,
    }
}
```

<details>
<summary>정답 보기</summary>

`Injects`를 다루지 않았기 때문입니다.

`match`는 모든 경우를 다뤄야 합니다. `Injects`가 들어왔을 때 무엇을 돌려줄지
정해지지 않았으므로 컴파일러가 거부합니다.

고치는 방법은 두 가지입니다.

```rust
// 명시적으로 적습니다. 나중에 값을 추가하면 여기서 오류가 나므로 안전합니다.
EdgeKind::Injects => false,

// 나머지를 묶습니다. 편하지만 값을 추가해도 알려 주지 않습니다.
_ => false,
```

열거형에는 첫 번째를 권합니다.

</details>

### 문제 2 [고치기]

```bash
cd book/exercises
cargo test -p ex_03_01_a
```

빠뜨린 경우를 채우는 문제입니다.

### 문제 3 [쓰기]

```bash
cargo test -p ex_03_01_b
```

가드를 써서 후보 개수에 따라 분기하는 함수를 작성하는 문제입니다.

## 정리

`match`는 모든 경우를 다뤄야 하며, 빠뜨리면 컴파일되지 않습니다. 열거형에 값을
추가하면 고쳐야 할 곳을 컴파일러가 전부 찾아 줍니다.

열거형이 품은 값을 그 자리에서 꺼낼 수 있고, 없는 경우에 쓰려고 하면
컴파일되지 않습니다.

`_`는 나머지를 묶지만 열거형에는 되도록 쓰지 않는 편이 낫습니다. 빠짐없음
검사의 이득이 사라지기 때문입니다.

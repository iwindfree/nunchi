# 4.3 `map`, `filter`, `collect` 체인 읽기

> **선행 장**: [4.2 이터레이터](04-2-iterators.md), [2.1 `Option<T>`](02-1-option.md)
> **연습문제**: 6개

nunchi 코드에서 가장 자주 만나게 될 형태입니다. 여러 메서드를 점으로 이어
붙인 긴 줄인데, 읽는 요령이 있습니다.

## 읽는 순서는 위에서 아래입니다

```rust
let langs: Vec<String> = files
    .iter()                          // ① 파일을 하나씩 꺼냅니다
    .filter(|f| f.is_code())         // ② 코드 파일만 남깁니다
    .map(|f| f.language.clone())     // ③ 언어 이름만 뽑습니다
    .collect();                      // ④ 목록으로 모읍니다
```

각 줄이 한 단계입니다. **값이 위에서 아래로 전달됩니다.**

```
files ──▶ [하나씩] ──▶ [코드만] ──▶ [언어 이름] ──▶ Vec<String>
```

읽을 때는 마지막 줄을 먼저 보는 편이 낫습니다. `.collect()`면 목록을
만드는 것이고, `.count()`면 개수를 세는 것이고, `.any()`면 조건 확인입니다.
**무엇을 만들려는지 알고 나서 중간 단계를 보면 훨씬 쉽습니다.**

## `.collect()`의 타입은 어디서 오는가

`.collect()`는 여러 종류를 만들 수 있습니다. `Vec`도 되고 `HashMap`도 되고
`String`도 됩니다. 그래서 **어느 것을 만들지 알려 줘야 합니다.**

방법이 두 가지입니다.

```rust
// 변수에 타입을 적습니다
let langs: Vec<String> = files.iter().map(|f| f.lang.clone()).collect();

// collect 에 직접 적습니다
let langs = files.iter().map(|f| f.lang.clone()).collect::<Vec<String>>();
```

`::<...>`를 터보피시(turbofish)라고 부릅니다. nunchi 코드에 자주 나옵니다.

```rust
// crates/nunchi-core/src/store/sqlite.rs 에서
Ok(rows.collect::<Result<Vec<_>, _>>()?)
```

`Vec<_>`의 `_`는 "컴파일러가 알아서 정하라"는 뜻입니다. 안에 무엇이 들어가는지
이미 알 수 있으므로 적지 않아도 됩니다.

## `Result`가 섞인 체인

이 형태가 처음 보면 어렵습니다.

```rust
rows.collect::<Result<Vec<_>, _>>()?
```

`rows`는 `Result`를 하나씩 꺼내 줍니다. 데이터베이스에서 행을 읽는 중에
실패할 수 있기 때문입니다.

```
Result<Node>, Result<Node>, Result<Node>, ...
```

이것을 `Result<Vec<Node>>`로 모읍니다. **하나라도 실패하면 전체가
실패입니다.** 성공한 것들만 모으는 것이 아니라 전부 성공해야 목록이
나옵니다.

`.collect()`가 이 변환을 해 줍니다. 그리고 `?`로 실패를 위로 올립니다.

```rust
// 이런 뜻입니다
Vec<Result<Node>>  →  Result<Vec<Node>>  →  Vec<Node> (실패하면 함수를 끝냅니다)
```

## `filter_map`

`filter`와 `map`을 한 번에 합친 것입니다. `Option`을 돌려주는 함수를 넘기면,
`Some`인 것만 남기고 값을 꺼내 줍니다.

```rust
// 두 단계로 쓰면 이렇습니다
.map(|s| s.parse::<u32>().ok())      // Option<u32> 가 됩니다
.filter(|o| o.is_some())              // Some 만 남깁니다
.map(|o| o.unwrap())                  // 값을 꺼냅니다

// filter_map 이면 한 줄입니다
.filter_map(|s| s.parse::<u32>().ok())
```

nunchi에서 자주 쓰입니다.

```rust
// crates/nunchi-core/src/pack.rs 에서
pub fn repo_roots(config: &crate::Config) -> HashMap<String, std::path::PathBuf> {
    config
        .solution
        .repos
        .iter()
        .filter_map(|p| {
            let canonical = p.canonicalize().ok()?;
            let name = canonical.file_name()?.to_string_lossy().to_string();
            Some((name, canonical))
        })
        .collect()
}
```

한 줄씩 읽습니다.

- `.iter()`로 저장소 경로를 하나씩 꺼냅니다.
- 클로저 안에서 `?`를 씁니다. 경로를 정규화할 수 없거나 이름을 얻을 수
  없으면 그 자리에서 `None`을 돌려주고, `filter_map`이 그것을 버립니다.
- 성공하면 `(이름, 경로)` 짝을 `Some`으로 감싸 돌려줍니다.
- `.collect()`가 `HashMap`으로 모읍니다. 짝을 모으면 `HashMap`이 됩니다.

**클로저 안에서 `?`를 쓸 수 있다는 점이 중요합니다.** 클로저도 함수이므로
`Option`을 돌려주면 `?`가 동작합니다.

## 실제 코드 읽어 보기

nunchi에서 가장 긴 체인을 하나 읽습니다.

```rust
// crates/nunchi-core/src/index.rs 에서
let file_mtime = meta
    .modified()
    .ok()
    .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
    .map(|d| d.as_secs() as i64);
```

[3.2장](03-2-if-let.md)에서 봤던 중첩 `if let`을 이 형태로 고친 것입니다.

- `meta.modified()`는 `Result<SystemTime>`입니다. 파일 시스템이 수정 시각을
  지원하지 않을 수 있습니다.
- `.ok()`로 `Option<SystemTime>`으로 바꿉니다. 왜 실패했는지는 알 필요가
  없습니다.
- `.and_then(...)`은 값이 있을 때만 다음 계산을 합니다. 그 계산도 `Option`을
  돌려주므로 `.map`이 아니라 `.and_then`입니다.
- `.map(...)`으로 초 단위 정수로 바꿉니다.

결과는 `Option<i64>`입니다. 어느 단계에서든 실패하면 `None`이 되고, 그러면
최근성 점수를 0으로 둡니다.

**`.map`과 `.and_then`의 차이**가 여기서 드러납니다. 넘긴 함수가 보통 값을
돌려주면 `.map`, `Option`을 돌려주면 `.and_then`입니다. `.map`을 쓰면
`Option<Option<T>>`처럼 두 겹이 됩니다.

## 언제 체인을 쓰지 않는가

체인이 항상 낫지는 않습니다.

**부수 효과가 있으면 반복문이 낫습니다.** 통계를 갱신하거나 여러 목록에
나눠 담는 경우입니다.

```rust
// crates/nunchi-core/src/index.rs 에서
for (repo, seen) in &seen_by_repo {
    stats.pruned += store.prune_missing_files(repo, seen)?;
}
```

`stats`를 바꾸고 `?`로 오류를 올립니다. 체인으로 쓰면 오히려 어려워집니다.

**단계가 다섯을 넘으면 나눕니다.** 중간 결과에 이름을 붙이는 편이 낫습니다.

## 연습문제

### 문제 1 [읽기]

아래 체인이 무엇을 하는지 한 문장으로 말하십시오.

```rust
let count = nodes
    .iter()
    .filter(|n| n.kind == NodeKind::Symbol)
    .filter(|n| n.path.is_some())
    .count();
```

<details>
<summary>정답 보기</summary>

경로가 있는 심볼 노드의 개수를 셉니다.

읽는 요령대로 마지막 줄부터 봅니다. `.count()`이므로 개수를 세는 것이고,
그 앞의 두 `filter`가 어떤 것을 세는지 정합니다.

참고로 두 `filter`는 하나로 합칠 수 있습니다.

```rust
.filter(|n| n.kind == NodeKind::Symbol && n.path.is_some())
```

나누어 쓰면 각 조건이 무엇인지 잘 보이고, 합치면 짧습니다. 조건이 둘까지는
나누는 편이 읽기 좋습니다.

</details>

### 문제 2 [읽기]

`.map`과 `.and_then` 중 무엇을 써야 합니까?

```rust
// (가) 문자열의 길이를 구한다
opt_name.???(|n| n.len())

// (나) 문자열을 숫자로 바꾼다. 실패할 수 있다
opt_text.???(|t| t.parse::<u32>().ok())
```

<details>
<summary>정답 보기</summary>

(가)는 `.map`입니다. `n.len()`이 보통 값(`usize`)을 돌려줍니다.

(나)는 `.and_then`입니다. `parse().ok()`가 `Option<u32>`를 돌려주므로,
`.map`을 쓰면 `Option<Option<u32>>`가 되어 두 겹이 됩니다.

기준은 간단합니다. **넘긴 함수가 `Option`을 돌려주면 `.and_then`입니다.**

</details>

### 문제 3 [고치기]

```bash
cd book/exercises
cargo test -p ex_04_03_a
```

`.map`을 `.and_then`으로 바꿔야 하는 문제입니다.

### 문제 4 [쓰기]

```bash
cargo test -p ex_04_03_b
```

`filter_map`으로 체인을 줄이는 문제입니다.

### 문제 5 [쓰기]

```bash
cargo test -p ex_04_03_c
```

`collect`로 `HashMap`을 만드는 문제입니다.

### 문제 6 [고치기]

```bash
cargo test -p ex_04_03_d
```

`Result`가 섞인 체인을 다루는 문제입니다.

## 정리

체인은 위에서 아래로 읽되 마지막 줄을 먼저 보십시오. 무엇을 만들려는지 알고
나면 중간 단계가 쉬워집니다.

`.collect()`는 만들 타입을 알려 줘야 하며, 변수에 적거나 터보피시를 씁니다.
`Result`가 섞이면 `Result<Vec<_>, _>`로 모으고 하나라도 실패하면 전체가
실패합니다.

넘긴 함수가 `Option`을 돌려주면 `.map`이 아니라 `.and_then`입니다.

부수 효과가 있거나 단계가 다섯을 넘으면 반복문이나 중간 변수가 낫습니다.

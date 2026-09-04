# 7.2 테스트를 같은 파일에 두는 관례

> **선행 장**: [7.1 모듈과 가시성](07-1-modules.md), [5.4 `#[derive]`와 serde 속성](05-4-derive.md)
> **연습문제**: 1개

Rust는 테스트를 소스 파일 안에 씁니다. 다른 언어와 다른 점입니다.

## 파일 아래에 붙입니다

```rust
// crates/nunchi-core/src/path.rs 에서
pub fn normalize(path: &Path) -> String {
    // 실제 코드입니다
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_backslashes() {
        assert_eq!(normalize(Path::new(r"src\main\java\App.java")), "src/main/java/App.java");
    }
}
```

`#[cfg(test)]`는 "테스트를 빌드할 때만 포함한다"는 뜻입니다. `cargo build`로
만드는 실행 파일에는 들어가지 않습니다.

`use super::*`는 바깥 모듈의 것을 전부 가져옵니다. 테스트가 하위 모듈이므로
바깥에 있는 함수를 쓰려면 필요합니다.

## 비공개 함수도 테스트할 수 있습니다

같은 파일 안에 있으므로 `pub`이 아닌 함수도 부를 수 있습니다.

```rust
// crates/nunchi-core/src/pack.rs 에서
fn recency_score(mtime: Option<i64>, now: i64) -> f32 {
    // pub 이 아닙니다
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recency_decays_with_half_life() {
        let now = 1_000_000_000i64;
        let day = 86_400i64;
        assert!((recency_score(Some(now), now) - 1.0).abs() < 0.01, "오늘 = 1.0");
        assert!((recency_score(Some(now - 30 * day), now) - 0.5).abs() < 0.01, "30일 = 0.5");
    }
}
```

**이것이 파일 안에 두는 가장 큰 이유입니다.** 내부 함수를 공개하지 않고도
테스트할 수 있습니다.

## 확인하는 방법

| 매크로 | 하는 일 |
|---|---|
| `assert!(cond)` | 참인지 확인합니다 |
| `assert_eq!(a, b)` | 같은지 확인합니다 |
| `assert_ne!(a, b)` | 다른지 확인합니다 |

메시지를 덧붙일 수 있습니다.

```rust
assert!(
    names.contains(&"find_order"),
    "심볼을 찾지 못했습니다: {names:?}"
);
```

**실패했을 때 무엇이 잘못됐는지 알려 주므로 메시지를 붙이는 편이 낫습니다.**
`assert!(x)`만 있으면 어떤 값이었는지 알 수 없습니다.

`assert_eq!`는 실패하면 양쪽 값을 자동으로 출력합니다. 그러려면 `Debug`가
있어야 합니다([5.4장](05-4-derive.md)).

## 실패를 확인하는 테스트

`Result`를 돌려주는 테스트를 쓸 수 있습니다.

```rust
#[test]
fn upsert_is_idempotent() -> Result<()> {
    let mut store = SqliteStore::open_in_memory()?;
    let node = sample_file("api", "src/OrderService.java", "java");
    store.upsert_nodes(std::slice::from_ref(&node))?;
    store.upsert_nodes(std::slice::from_ref(&node))?;
    assert_eq!(store.count_nodes()?, 1);
    Ok(())
}
```

`?`를 쓸 수 있어서 편합니다. 오류가 나면 테스트가 실패합니다.

## 도우미 함수

테스트에서만 쓰는 함수를 `mod tests` 안에 둡니다.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_file(repo: &str, path: &str, lang: &str) -> Node {
        let mut n = Node::new(NodeId::file(repo, path), NodeKind::File, path, repo);
        n.path = Some(path.to_string());
        n.lang = Some(lang.to_string());
        n
    }

    #[test]
    fn search_finds_by_name() -> Result<()> {
        let mut store = SqliteStore::open_in_memory()?;
        store.upsert_nodes(&[
            sample_file("api", "src/OrderService.java", "java"),
            sample_file("web", "src/hooks/useOrder.ts", "typescript"),
        ])?;
        // ...
    }
}
```

테스트마다 노드를 만드는 코드를 반복하지 않아도 됩니다.

## 별도 파일에 두는 테스트

`tests/` 디렉터리에 두면 **밖에서 쓰는 것처럼** 테스트합니다. 공개된 것만
쓸 수 있습니다.

```
crates/nunchi-core/
├── src/
│   └── lib.rs
└── tests/
    └── integration.rs      공개 API 만 씁니다
```

nunchi 본체는 이것을 쓰지 않고 파일 안 테스트만 씁니다. 이 책의 연습문제는
`tests/`를 쓰는데, 푸는 사람이 공개 API를 제대로 만들었는지 확인하기
위해서입니다.

## nunchi의 테스트가 보장하는 것

```bash
cargo test              # 전체
cargo test -p nunchi-core framework    # 모듈별
```

73개가 있으며 각각 다른 것을 보장합니다.

| 테스트 | 잡아내는 문제 |
|---|---|
| `all_queries_compile` | tree-sitter 쿼리의 잘못된 노드 타입 |
| `route_definitions_are_not_client_calls` | 라우트 정의를 호출로 오인 |
| `normalizes_all_three_param_syntaxes` | 경로 표기 세 가지가 같은 값이 되는지 |
| `pagerank_concentrates_near_seeds` | 시드 지배력과 거리 감쇠 |
| `prune_removes_vanished_files_and_their_edges` | 사라진 파일 정리 |

`all_queries_compile`이 특히 중요합니다. tree-sitter 쿼리에 잘못된 노드 타입을
적으면 컴파일 시점이 아니라 **실행 시점에** 오류가 납니다. 이 테스트가 그것을
미리 잡습니다.

## 연습문제

### 문제 1 [쓰기]

```bash
cd book/exercises
cargo test -p ex_07_02_a
```

파일 안 테스트를 작성하는 문제입니다.

## 정리

Rust는 테스트를 소스 파일 안에 `#[cfg(test)] mod tests`로 씁니다. 비공개
함수도 테스트할 수 있는 것이 가장 큰 이득입니다.

`assert!`에는 메시지를 붙이는 편이 낫습니다. 실패했을 때 무엇이 잘못됐는지
알 수 있기 때문입니다.

테스트가 `Result`를 돌려주게 만들면 `?`를 쓸 수 있습니다.

`tests/` 디렉터리에 두면 공개된 것만 쓸 수 있으므로 밖에서 쓰는 것처럼
확인하게 됩니다.

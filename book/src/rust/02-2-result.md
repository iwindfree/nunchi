# 2.2 `Result<T, E>`

> **선행 장**: [2.1 `Option<T>`](02-1-option.md)
> **연습문제**: 2개

`Option`이 "값이 없을 수 있다"를 나타낸다면, `Result`는 "실패할 수 있다"를
나타냅니다. nunchi 코드에 106번 나옵니다.

## 예외가 없습니다

Java나 Python은 예외를 던집니다. 함수 서명만 보면 그 함수가 실패할 수 있는지
알기 어렵고, 어디서 잡아야 하는지도 흐름을 따라가야 압니다.

Rust에는 예외가 없습니다. 실패할 수 있으면 **반환 타입에 적습니다.**

```rust
fn load(path: &Path) -> Result<Config, Error>
```

이 함수는 성공하면 `Config`를, 실패하면 `Error`를 돌려줍니다. 부르는 쪽은
두 경우를 모두 처리해야 합니다.

## `Result`도 열거형입니다

```rust
enum Result<T, E> {
    Ok(T),          // 성공했으며 결과를 품고 있습니다
    Err(E),         // 실패했으며 오류를 품고 있습니다
}
```

`Option`과 구조가 같습니다. 다른 점은 실패한 쪽에도 정보가 담긴다는 것입니다.
`None`은 왜 없는지 알려 주지 않지만 `Err(e)`는 무엇이 잘못됐는지 알려 줍니다.

```rust
let ok: Result<u32, String> = Ok(3);
let bad: Result<u32, String> = Err("파일을 읽을 수 없습니다".to_string());
```

## 언제 무엇을 쓰는가

| 상황 | 타입 |
|---|---|
| 값이 없는 것이 정상입니다 | `Option<T>` |
| 실패했고 이유를 알려야 합니다 | `Result<T, E>` |

예를 들어 이렇습니다.

```rust
// 확장자가 없는 파일은 정상입니다. 실패가 아닙니다.
fn extension(path: &str) -> Option<&str>

// 파일을 못 읽으면 실패입니다. 왜 못 읽었는지 알려야 합니다.
fn read_file(path: &Path) -> Result<String, io::Error>
```

`Makefile`처럼 확장자 없는 파일이 있는 것은 자연스럽습니다. 반면 파일을 못
읽는 것은 권한 문제인지 없는 파일인지 알아야 대응할 수 있습니다.

## 결과를 다루는 방법

`Option`과 비슷한 메서드가 있습니다.

| 메서드 | 하는 일 |
|---|---|
| `.unwrap_or(v)` | 실패하면 `v`를 씁니다 |
| `.unwrap_or_default()` | 실패하면 기본값을 씁니다 |
| `.map(f)` | 성공했으면 `f`를 적용합니다 |
| `.ok()` | `Result`를 `Option`으로 바꿉니다. 오류 정보는 버립니다 |
| `.is_ok()`, `.is_err()` | 성공했는지 실패했는지만 확인합니다 |
| `?` | 실패하면 함수를 즉시 끝냅니다 |

`.ok()`가 nunchi에서 자주 쓰입니다.

```rust
// crates/nunchi-core/src/index.rs 에서
let file_mtime = meta
    .modified()
    .ok()
    .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
    .map(|d| d.as_secs() as i64);
```

파일의 수정 시각을 읽습니다. 세 단계 모두 실패할 수 있는데, **실패해도
괜찮습니다.** 수정 시각을 모르면 최근성 점수를 0으로 두면 되기 때문입니다.

그래서 `Result`를 `.ok()`로 `Option`으로 바꿔 버립니다. 오류 이유를 알 필요가
없다는 뜻입니다. 이렇게 하면 세 단계를 한 줄로 이을 수 있습니다.

## `Result`를 무시하면 경고가 납니다

```rust
fn might_fail() -> Result<(), Error> { ... }

might_fail();          // 경고가 납니다
```

컴파일러가 "이 결과를 확인하지 않았다"고 알려 줍니다. 실패를 조용히 넘기는
실수를 막습니다.

정말로 무시하고 싶으면 그렇다고 적어야 합니다.

```rust
let _ = might_fail();
```

nunchi에도 이런 자리가 있습니다.

```rust
// crates/nunchi-core/src/cache.rs 에서
let _ = self.conn.execute(
    "UPDATE extract_cache SET used_at = strftime('%s','now') WHERE hash = ?1",
    params![hash],
);
```

캐시의 마지막 사용 시각을 갱신하는 질의입니다. 실패해도 캐시 동작에는 영향이
없으므로 무시합니다. `let _ =`가 "일부러 무시했다"는 표시가 됩니다.

## 이 프로젝트에서는

nunchi의 거의 모든 함수가 `Result`를 돌려줍니다.

```rust
// crates/nunchi-core/src/index.rs 에서
pub fn index_all(config: &Config, store: &mut SqliteStore) -> Result<IndexStats>
```

오류 타입이 안 보이는데, `anyhow::Result`를 쓰기 때문입니다.

```rust
type Result<T> = std::result::Result<T, anyhow::Error>;
```

오류 타입을 매번 적지 않아도 되게 만든 별칭입니다. [2.4장](02-4-anyhow.md)에서
다룹니다.

## 연습문제

### 문제 1 [읽기]

아래 함수들의 반환 타입으로 `Option`과 `Result` 중 무엇이 맞습니까?

```rust
// (가) 심볼 이름으로 정의 위치를 찾는다. 없는 이름일 수 있다.
// (나) 설정 파일을 읽어 구조체로 만든다.
// (다) 문자열에서 확장자를 꺼낸다.
// (라) SQLite 에 노드를 저장한다.
```

<details>
<summary>정답 보기</summary>

(가)는 `Option`입니다. 찾는 이름이 없는 것은 정상이며 실패가 아닙니다.

(나)는 `Result`입니다. 파일이 없거나, 읽을 권한이 없거나, 내용이 잘못됐을 수
있습니다. 각각 다른 대응이 필요하므로 이유를 알려야 합니다.

(다)는 `Option`입니다. 확장자 없는 파일은 정상입니다.

(라)는 `Result`입니다. 데이터베이스 오류가 날 수 있고 그 이유를 알아야 합니다.

</details>

### 문제 2 [고치기]

```bash
cd book/exercises
cargo test -p ex_02_02_a
```

`Result`를 처리하지 않아 경고가 나는 코드를 고치는 문제입니다.

## 정리

Rust에는 예외가 없고 실패를 반환 타입에 적습니다. `Result<T, E>`는 성공하면
`Ok(T)`, 실패하면 `Err(E)`입니다.

값이 없는 것이 정상이면 `Option`, 실패했고 이유를 알려야 하면 `Result`를
씁니다.

`Result`를 무시하면 경고가 납니다. 일부러 무시할 때는 `let _ =`로 표시합니다.

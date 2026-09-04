# 2.4 `anyhow`와 오류 관례

> **선행 장**: [2.3 `?` 연산자](02-3-question-mark.md)
> **연습문제**: 2개

앞 장에서 `Result<Config>`처럼 오류 타입이 안 보이는 서명을 봤습니다. 이 장에서
그 이유를 설명합니다.

## 오류 타입을 직접 만드는 부담

표준 방식대로 하면 프로젝트마다 오류 타입을 정의해야 합니다.

```rust
enum NunchiError {
    Io(std::io::Error),
    Toml(toml::de::Error),
    Sqlite(rusqlite::Error),
    TreeSitter(tree_sitter::LanguageError),
    // 라이브러리를 추가할 때마다 늘어납니다
}
```

그리고 각 오류를 이 타입으로 바꾸는 코드도 써야 합니다. 라이브러리를 하나 더
쓰면 또 늘어납니다.

**애플리케이션에서는 이 부담을 감수할 만한 이득이 없는 경우가 많습니다.** 오류를
종류별로 다르게 처리하지 않고 그냥 사용자에게 보여 주고 끝내기 때문입니다.

## `anyhow`가 하는 일

`anyhow`는 "어떤 오류든 담을 수 있는 오류 타입" 하나를 제공합니다.

```rust
use anyhow::Result;

fn load(path: &Path) -> Result<Config> {
    let text = std::fs::read_to_string(path)?;    // io::Error 를 담습니다
    let config = toml::from_str(&text)?;          // toml::Error 를 담습니다
    Ok(config)
}
```

`anyhow::Result<T>`는 `std::result::Result<T, anyhow::Error>`의 별칭입니다.
오류 타입을 매번 적지 않아도 됩니다.

## 맥락을 덧붙입니다

`anyhow`가 유용한 이유가 여기에 있습니다. 오류에 설명을 덧붙일 수 있습니다.

```rust
// crates/nunchi-core/src/config.rs 에서
pub fn load(path: &Path) -> Result<Self> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("설정 파일을 읽을 수 없습니다: {}", path.display()))?;
    let mut config: Config = toml::from_str(&text)
        .with_context(|| format!("설정 파일 파싱 실패: {}", path.display()))?;
    // ...
}
```

`with_context`가 없으면 사용자는 이런 메시지만 봅니다.

```
Error: No such file or directory (os error 2)
```

어떤 파일을 못 찾았는지 알 수 없습니다. `with_context`를 붙이면 이렇게 됩니다.

```
Error: 설정 파일을 읽을 수 없습니다: /Users/me/dev/nunchi.toml

Caused by:
    No such file or directory (os error 2)
```

**맥락이 쌓이면서 어디서 무엇을 하다 실패했는지 드러납니다.**

`with_context`는 클로저를 받습니다. 오류가 났을 때만 메시지를 만들기
위해서입니다. `.context("고정 문자열")`도 있는데, 이쪽은 메시지를 항상
만듭니다.

## 오류를 직접 만듭니다

`bail!`은 그 자리에서 오류를 만들어 함수를 끝냅니다.

```rust
// crates/nunchi-cli/src/main.rs 에서
if !db_path.exists() {
    anyhow::bail!("인덱스가 없습니다. `nunchi index`를 먼저 실행하세요.");
}
```

`return Err(anyhow!("..."))`와 같은 뜻이며 더 짧습니다.

nunchi에서 `bail!`을 쓰는 자리에는 공통점이 있습니다. **무엇을 해야 하는지
함께 알려 줍니다.**

```rust
anyhow::bail!(
    "인덱스 스키마 버전 불일치 (인덱스={v}, 기대={SCHEMA_VERSION}). \
     `nunchi index --rebuild`로 재구축하세요."
);
```

오류 메시지가 문제만 알려 주고 해결책을 안 알려 주면 사용자가 다시 찾아봐야
합니다.

## 라이브러리에서는 다릅니다

`anyhow`는 애플리케이션용입니다. 라이브러리를 만든다면 `thiserror`로 구체적인
오류 타입을 정의하는 편이 낫습니다. 쓰는 쪽에서 오류 종류에 따라 다르게
처리할 수 있어야 하기 때문입니다.

nunchi는 애플리케이션이므로 `anyhow`를 씁니다. `Cargo.toml`에 `thiserror`가
들어 있기는 한데 아직 쓰지 않습니다.

## 이 프로젝트에서 오류를 다루는 방식

세 가지 방식이 상황에 따라 쓰입니다.

**첫째, 위로 올립니다.** 회복할 수 없는 오류입니다.

```rust
let store = SqliteStore::open(&db_path)?;
```

인덱스를 못 열면 아무것도 할 수 없으므로 그대로 올립니다.

**둘째, 건너뜁니다.** 한 항목의 실패가 전체를 막지 않아야 할 때입니다.

```rust
// crates/nunchi-core/src/index.rs 에서
let Ok(entry) = entry else { continue };
```

**셋째, 기록하고 계속합니다.** 알아야 하지만 멈출 이유는 없을 때입니다.

```rust
Err(e) => {
    tracing::warn!("추출 실패 {rel}: {e}");
    continue;
}
```

파일 하나를 파싱하지 못했다는 사실은 알아야 하지만, 나머지 파일은 계속
처리해야 합니다.

## 연습문제

### 문제 1 [읽기]

아래 두 오류 메시지 중 어느 쪽이 나은지 판단하고 이유를 말하십시오.

```
(가) Error: invalid digit found in string

(나) Error: 설정 파일의 max_commits 값을 읽을 수 없습니다: nunchi.toml:12
     Caused by: invalid digit found in string
```

<details>
<summary>정답 보기</summary>

(나)가 낫습니다.

(가)는 무엇이 잘못됐는지는 알려 주지만 **어디서** 잘못됐는지 알려 주지
않습니다. 설정 파일이 수십 줄이면 어느 값이 문제인지 찾아야 합니다.

(나)는 파일 이름과 줄 번호, 문제가 된 설정 항목까지 알려 줍니다.
`with_context`가 이런 맥락을 덧붙이는 도구입니다.

원래 오류도 `Caused by:` 아래에 남아 있다는 점이 중요합니다. 맥락을 덧붙여도
근본 원인이 사라지지 않습니다.

</details>

### 문제 2 [고치기]

```bash
cd book/exercises
cargo test -p ex_02_04_a
```

오류에 맥락을 덧붙이는 문제입니다.

## 정리

`anyhow`는 어떤 오류든 담는 타입 하나를 제공하여 오류 타입을 매번 정의하는
부담을 없앱니다. 애플리케이션에 적합하며 라이브러리에는 `thiserror`가 낫습니다.

`with_context`로 맥락을 덧붙이면 어디서 무엇을 하다 실패했는지 드러납니다.
`bail!`은 그 자리에서 오류를 만들어 함수를 끝냅니다.

오류를 만났을 때 위로 올릴지, 건너뛸지, 기록하고 계속할지 판단하는 것이
중요합니다.

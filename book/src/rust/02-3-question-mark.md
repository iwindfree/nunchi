# 2.3 `?` 연산자

> **선행 장**: [2.2 `Result<T, E>`](02-2-result.md)
> **연습문제**: 3개

nunchi 코드에 `?`가 **319번** 나옵니다. 파일 23개 중 18개에 있습니다. 이것을
모르면 코드를 한 줄도 읽기 어렵습니다.

## 무엇을 대신하는가

`Result`를 돌려주는 함수를 부를 때마다 성공과 실패를 확인해야 합니다. 그것을
직접 쓰면 이렇게 됩니다.

```rust
fn load(path: &Path) -> Result<Config> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => return Err(e.into()),
    };
    let config = match toml::from_str(&text) {
        Ok(c) => c,
        Err(e) => return Err(e.into()),
    };
    Ok(config)
}
```

실제 일은 두 줄인데 오류 처리가 절반을 차지합니다. `?`를 쓰면 이렇게 줄어듭니다.

```rust
fn load(path: &Path) -> Result<Config> {
    let text = std::fs::read_to_string(path)?;
    let config = toml::from_str(&text)?;
    Ok(config)
}
```

**`?`가 하는 일은 한 문장으로 설명됩니다.**

> 성공했으면 값을 꺼내서 계속 진행하고, 실패했으면 그 오류를 그대로 돌려주며
> 함수를 즉시 끝냅니다.

## 규칙 하나

`?`는 **`Result`를 돌려주는 함수 안에서만** 쓸 수 있습니다.

```rust
fn ok() -> Result<Config> {
    let text = read_to_string(path)?;    // 됩니다
    // ...
}

fn bad() -> Config {
    let text = read_to_string(path)?;    // 컴파일 오류입니다
    // ...
}
```

당연합니다. `?`가 실패했을 때 오류를 돌려주려면 함수의 반환 타입이 `Result`여야
합니다.

`main` 함수도 `Result`를 돌려주게 만들 수 있습니다.

```rust
// crates/nunchi-cli/src/main.rs 에서
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init { repos, name, force } => cmd_init(repos, name, force),
        // ...
    }
}
```

`Result<()>`에서 `()`는 "돌려줄 값이 없다"는 뜻입니다. 성공하면 아무것도
돌려주지 않고 실패하면 오류를 돌려줍니다.

## `Option`에도 쓸 수 있습니다

`Option`을 돌려주는 함수 안에서는 `Option`에 `?`를 쓸 수 있습니다.

```rust
// crates/nunchi-core/src/lang.rs 에서
pub fn detect(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    Some(match ext.as_str() {
        "java" => "java",
        "rs" => "rust",
        _ => return None,
    })
}
```

`extension()`이 `None`이면 거기서 함수가 끝나고 `None`을 돌려줍니다.
`to_str()`도 마찬가지입니다.

**`Result`와 `Option`을 섞을 수는 없습니다.** `Result`를 돌려주는 함수 안에서
`Option`에 `?`를 붙이면 컴파일되지 않습니다. 그럴 때는 `.ok_or()`로 먼저
변환합니다.

## 오류 타입이 자동으로 바뀝니다

`?`에는 설명이 하나 더 필요합니다. 아래 코드를 보십시오.

```rust
fn load(path: &Path) -> Result<Config> {
    let text = std::fs::read_to_string(path)?;    // io::Error 가 납니다
    let config = toml::from_str(&text)?;          // toml::Error 가 납니다
    Ok(config)
}
```

두 함수가 **서로 다른 오류 타입**을 돌려줍니다. 그런데 이 함수의 반환 타입은
하나입니다. 어떻게 되는 것입니까?

`?`가 오류를 돌려줄 때 **함수의 오류 타입으로 자동 변환합니다.** 그래서 서로
다른 오류를 한 함수에서 섞어 쓸 수 있습니다.

> 이 변환이 어떻게 가능한지는 `From` 트레이트 때문입니다.
> 지금은 **"자동으로 변환된다"는 사실만 받아들이시면 됩니다.**
> 원리는 [5.3장 `From`과 `Into`](05-3-from-into.md)에서 설명합니다.
> 트레이트를 아직 다루지 않았기 때문입니다.

## 이 프로젝트에서는

`?`가 연속으로 나오는 실제 코드를 봅니다.

```rust
// crates/nunchi-core/src/store/sqlite.rs 에서
pub fn prune_missing_files(&mut self, repo: &str, seen_paths: &[String]) -> Result<usize> {
    let tx = self.conn.transaction()?;
    tx.execute_batch("CREATE TEMP TABLE IF NOT EXISTS seen_keys (k TEXT PRIMARY KEY)")?;
    tx.execute("DELETE FROM seen_keys", [])?;
    {
        let mut ins = tx.prepare("INSERT OR IGNORE INTO seen_keys (k) VALUES (?1)")?;
        for p in seen_paths {
            ins.execute(params![compare_key(p)])?;
        }
    }
    // ...
    tx.commit()?;
    Ok(doomed.len())
}
```

`?`가 여섯 번 나옵니다. 데이터베이스 작업은 어느 단계에서든 실패할 수 있는데,
**실패하면 그 자리에서 함수가 끝나고 오류가 위로 올라갑니다.**

`?`가 없었다면 각 줄마다 `match`를 써야 했고, 이 함수는 세 배로 길어졌을
것입니다. 그리고 실제 로직이 오류 처리에 파묻혀 읽기 어려워졌을 것입니다.

## `?` 뒤에 메서드를 이어 붙일 수 있습니다

```rust
// crates/nunchi-core/src/store/sqlite.rs 에서
let rows = stmt.query_map([], |r| r.get::<_, String>(0).map(NodeId))?;
Ok(rows.collect::<Result<Vec<_>, _>>()?)
```

두 번째 줄에서 `?`가 두 번 쓰입니다. `collect`가 `Result`를 돌려주고 거기에
`?`를 붙였습니다.

읽는 순서는 안에서 밖입니다. `rows.collect(...)`가 먼저 실행되고, 그 결과에
`?`가 적용되고, 성공한 값이 `Ok(...)`로 감싸집니다.

## `?`가 없는 곳

모든 실패를 위로 올리는 것이 항상 옳지는 않습니다.

```rust
// crates/nunchi-core/src/index.rs 에서
let Ok(bytes) = std::fs::read(npath::to_extended_length(abs)) else { continue };
```

파일 하나를 못 읽었다고 인덱싱 전체를 멈추면 안 됩니다. 그래서 `?`를 쓰지 않고
그 파일만 건너뜁니다. `let ... else`는 [3.3장](03-3-let-else.md)에서 다룹니다.

**언제 올리고 언제 건너뛸지 판단하는 것**이 오류 처리의 핵심입니다. `?`는 올릴
때 쓰는 도구일 뿐입니다.

## 연습문제

### 문제 1 [읽기]

아래 함수가 컴파일되지 않는 이유는 무엇입니까?

```rust
fn line_count(path: &Path) -> usize {
    let text = std::fs::read_to_string(path)?;
    text.lines().count()
}
```

<details>
<summary>정답 보기</summary>

반환 타입이 `usize`인데 `?`를 썼기 때문입니다.

`?`는 실패했을 때 오류를 돌려주는데, `usize`를 돌려주기로 한 함수는 오류를
돌려줄 수 없습니다.

고치는 방법은 두 가지입니다.

```rust
// 반환 타입을 Result 로 바꿉니다
fn line_count(path: &Path) -> Result<usize> {
    let text = std::fs::read_to_string(path)?;
    Ok(text.lines().count())
}

// 또는 실패했을 때 쓸 값을 정합니다
fn line_count(path: &Path) -> usize {
    std::fs::read_to_string(path)
        .map(|t| t.lines().count())
        .unwrap_or(0)
}
```

</details>

### 문제 2 [고치기]

```bash
cd book/exercises
cargo test -p ex_02_03_a
```

`match`로 길게 쓴 오류 처리를 `?`로 줄이는 문제입니다.

### 문제 3 [고치기]

```bash
cargo test -p ex_02_03_b
```

`Option`에 `?`를 쓰는 문제입니다.

## 정리

`?`는 성공하면 값을 꺼내 계속 진행하고, 실패하면 오류를 돌려주며 함수를 즉시
끝냅니다. `Result`나 `Option`을 돌려주는 함수 안에서만 쓸 수 있습니다.

오류 타입은 자동으로 변환되므로 서로 다른 오류를 한 함수에서 섞어 쓸 수
있습니다. 원리는 5.3장에서 다룹니다.

모든 실패를 위로 올리는 것이 항상 옳지는 않습니다. 파일 하나를 못 읽었다고
전체를 멈추면 안 되는 경우도 있습니다.

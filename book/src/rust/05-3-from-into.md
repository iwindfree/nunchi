# 5.3 `From`과 `Into`, 그리고 `?`의 나머지 절반

> **선행 장**: [5.2 트레이트](05-2-traits.md), [2.3 `?` 연산자](02-3-question-mark.md)
> **연습문제**: 2개

2.3장에서 미뤄 둔 설명을 여기서 마칩니다. `?`가 오류 타입을 자동으로 변환하는
원리가 이 장의 주제입니다.

## `From`은 변환 약속입니다

"A에서 B를 만들 수 있다"는 약속입니다.

```rust
impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        NodeId(s.to_string())
    }
}
```

이렇게 해 두면 `&str`에서 `NodeId`를 만들 수 있습니다.

```rust
let id = NodeId::from("file:api/A.java");
```

## `Into`는 자동으로 따라옵니다

`From`을 구현하면 `Into`가 공짜로 생깁니다. 방향만 반대입니다.

```rust
let id: NodeId = "file:api/A.java".into();
```

`.into()`가 `NodeId`를 만든다는 사실을 어떻게 압니까? **왼쪽에 적힌 타입을
보고 정합니다.** 그래서 `.into()`를 쓸 때는 결과 타입이 어딘가에 적혀 있어야
합니다.

nunchi 코드에 `.into()`가 자주 나옵니다.

```rust
// crates/nunchi-core/src/rules.rs 에서
RouteRule {
    lang: "java".into(),          // &str 에서 String 을 만듭니다
    annotation: anno.into(),
    method: method.into(),
    method_from_args_prefix: None,
    receivers: Vec::new(),
    method_from_args_list: None,
}
```

`lang` 필드가 `String`이므로 `.into()`가 `String`을 만듭니다.
`"java".to_string()`과 같은 뜻이며 더 짧습니다.

## `?`가 하는 일의 나머지 절반

이제 2.3장의 물음에 답할 수 있습니다. 이 코드가 왜 되는지 설명합니다.

```rust
fn load(path: &Path) -> Result<Config> {
    let text = std::fs::read_to_string(path)?;    // io::Error 가 납니다
    let config = toml::from_str(&text)?;          // toml::Error 가 납니다
    Ok(config)
}
```

서로 다른 오류 타입인데 반환 타입은 하나입니다.

**`?`는 실패했을 때 `From`을 써서 함수의 오류 타입으로 변환합니다.**

```rust
// ? 가 실제로 하는 일을 풀어 쓰면 이렇습니다
match std::fs::read_to_string(path) {
    Ok(v) => v,
    Err(e) => return Err(From::from(e)),    // 여기서 변환합니다
}
```

`anyhow::Error`는 거의 모든 오류 타입에서 `From`이 구현되어 있습니다. 그래서
어떤 오류든 받아 담을 수 있습니다.

이것이 [2.4장](02-4-anyhow.md)에서 말한 "어떤 오류든 담을 수 있는 타입"의
정체입니다. 특별한 장치가 아니라 `From` 구현이 많이 되어 있는 것뿐입니다.

## 함수 매개변수에 쓰는 `impl Into<T>`

이 형태를 라이브러리에서 보게 됩니다.

```rust
pub fn text(text: impl Into<String>) -> Self
```

"`String`으로 바꿀 수 있는 무엇이든 받는다"는 뜻입니다. `&str`도 되고
`String`도 됩니다. 부르는 쪽이 편해집니다.

nunchi의 `Node::new`가 이 형태입니다.

```rust
// crates/nunchi-core/src/model.rs 에서
pub fn new(id: NodeId, kind: NodeKind, name: impl Into<String>, repo: impl Into<String>) -> Self {
    Node {
        id,
        kind,
        name: name.into(),
        repo: repo.into(),
        // ...
    }
}
```

덕분에 부르는 쪽에서 `&str`이든 `String`이든 그냥 넘길 수 있습니다.

```rust
Node::new(id, NodeKind::File, &rel, repo)
```

`&rel`은 `&String`이고 `repo`는 `&str`인데 둘 다 그대로 넘어갑니다.
**`.to_string()`을 적을 필요가 없습니다.**

## `TryFrom`

변환이 실패할 수 있으면 `TryFrom`을 씁니다. `From`은 반드시 성공해야 합니다.

```rust
let n = u8::try_from(300u32);        // 실패합니다. 300 은 u8 에 안 들어갑니다
```

`TryFrom`은 `Result`를 돌려줍니다. nunchi에서는 쓰지 않습니다.

## 연습문제

### 문제 1 [쓰기]

```bash
cd book/exercises
cargo test -p ex_05_03_a
```

`From`을 구현하는 문제입니다.

### 문제 2 [고치기]

```bash
cargo test -p ex_05_03_b
```

`impl Into<String>`으로 매개변수를 받는 문제입니다.

## 정리

`From`은 "A에서 B를 만들 수 있다"는 약속이며, 구현하면 `Into`가 자동으로
따라옵니다.

`?`가 오류 타입을 자동 변환하는 원리가 `From`입니다. `anyhow::Error`가 거의
모든 오류에서 `From`을 구현하고 있으므로 어떤 오류든 담을 수 있습니다.

`impl Into<String>`을 매개변수에 쓰면 부르는 쪽에서 `&str`이든 `String`이든
그냥 넘길 수 있습니다.

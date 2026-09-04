# 3.3 `let ... else`와 `matches!`

> **선행 장**: [3.2 `if let`과 `while let`](03-2-if-let.md)
> **연습문제**: 2개

`let ... else`는 비교적 최근에 들어온 문법입니다. Rust 1.65부터 쓸 수 있으며,
책에 따라 안 나와 있을 수도 있습니다. nunchi 코드에 32번 나옵니다.

## 문제 상황

값을 꺼내서 **그 아래로 계속 진행해야 하는** 경우가 많습니다.

```rust
fn process(node: &Node) -> Option<String> {
    if let Some(path) = node.path.as_deref() {
        if let Some(span) = node.span {
            // 실제 로직이 여기서야 시작합니다
            Some(format!("{}:{}", path, span.start_line))
        } else {
            None
        }
    } else {
        None
    }
}
```

값을 두 개 꺼내려고 두 겹으로 들어갔습니다. 실제 로직은 한 줄인데 감싸는 코드가
여섯 줄입니다. 값이 셋이면 세 겹이 됩니다.

## `let ... else`가 평평하게 만듭니다

```rust
fn process(node: &Node) -> Option<String> {
    let Some(path) = node.path.as_deref() else { return None };
    let Some(span) = node.span else { return None };
    Some(format!("{}:{}", path, span.start_line))
}
```

읽는 방법은 이렇습니다. **"이 모양이면 값을 꺼내고, 아니면 `else` 블록을
실행한다"** 입니다.

들여쓰기가 늘지 않습니다. 값을 열 개 꺼내도 열 줄이 나란히 놓입니다.

## `else` 블록은 반드시 빠져나가야 합니다

`else` 안에서는 함수를 끝내거나 반복을 벗어나야 합니다.

```rust
let Some(x) = value else { return None };     // 됩니다
let Some(x) = value else { continue };        // 됩니다
let Some(x) = value else { break };           // 됩니다
let Some(x) = value else { panic!("없음") };  // 됩니다

let Some(x) = value else { 0 };               // 컴파일 오류입니다
```

마지막이 안 되는 이유가 있습니다. `else`를 지나 아래로 내려가면 `x`가 없는
상태가 되는데, 그 아래 코드는 `x`가 있다고 가정하고 있습니다. 그래서
컴파일러가 `else`에서 반드시 빠져나가도록 강제합니다.

## 이 프로젝트에서는

nunchi에서 가장 많이 쓰이는 자리는 반복 안에서 항목을 건너뛸 때입니다.

```rust
// crates/nunchi-core/src/index.rs 에서
for entry in walker {
    let Ok(entry) = entry else { continue };
    if !entry.file_type().is_some_and(|t| t.is_file()) {
        continue;
    }
    let abs = entry.path();
    let Some(rel) = npath::relative_to(root, abs) else { continue };
    // ...
    let Some(language) = lang::detect(abs) else { continue };
    let Ok(meta) = entry.metadata() else { continue };
    // ...
    let Ok(bytes) = std::fs::read(npath::to_extended_length(abs)) else { continue };
    let Ok(source) = std::str::from_utf8(&bytes) else {
        stats.files_skipped_binary += 1;
        continue;
    };
```

한 파일을 처리하면서 여섯 번 걸러 냅니다. 각 단계에서 실패하면 그 파일만
건너뛰고 다음 파일로 갑니다.

**`if let`으로 썼다면 여섯 겹으로 들어갔을 것입니다.** `let ... else` 덕분에
모두 같은 높이에 놓이고, 읽는 사람은 "이 조건들을 통과한 파일만 아래로
내려간다"고 이해하면 됩니다.

마지막 것은 `else` 블록에 두 줄이 들어 있습니다. 통계를 기록하고 건너뜁니다.
`else`가 블록이므로 여러 문장을 넣을 수 있습니다.

## `matches!`

값이 특정 모양인지 **참과 거짓으로만** 확인합니다. nunchi에 12번 나옵니다.

```rust
// match 로 쓰면 이렇습니다
let is_useful = match node.kind {
    NodeKind::Symbol | NodeKind::File | NodeKind::Route => true,
    _ => false,
};

// matches! 로 쓰면 한 줄입니다
let is_useful = matches!(node.kind, NodeKind::Symbol | NodeKind::File | NodeKind::Route);
```

실제 코드에서는 이렇게 쓰입니다.

```rust
// crates/nunchi-core/src/pack.rs 에서
if !matches!(node.kind, NodeKind::Symbol | NodeKind::File | NodeKind::Route) {
    continue;
}
```

팩에 담을 수 있는 노드 종류인지 확인합니다.

`matches!`에도 가드를 쓸 수 있습니다.

```rust
// crates/nunchi-core/src/framework.rs 에서
if !matches!(next_char, Some(c) if c.is_whitespace() || c == '>') {
    cursor = start + open.len();
    continue;
}
```

XML 태그를 읽으면서 `<select` 다음 글자가 공백이나 `>`인지 확인합니다.
`<selectKey>`를 `<select>`로 잘못 읽지 않기 위해서입니다.

## 세 문법을 언제 쓰는가

| 상황 | 문법 |
|---|---|
| 모든 경우를 다뤄야 한다 | `match` |
| 한 경우만 관심 있고 나머지는 무시한다 | `if let` |
| 값을 꺼내 아래로 계속 진행한다 | `let ... else` |
| 참과 거짓만 알면 된다 | `matches!` |
| 모양에 맞는 동안 반복한다 | `while let` |

## 연습문제

### 문제 1 [고치기]

```bash
cd book/exercises
cargo test -p ex_03_03_a
```

중첩된 `if let`을 `let ... else`로 바꾸는 문제입니다.

### 문제 2 [고치기]

```bash
cargo test -p ex_03_03_b
```

`match`를 `matches!`로 줄이는 문제입니다.

## 정리

`let ... else`는 값을 꺼내 아래로 계속 진행할 때 씁니다. 중첩이 늘지 않으므로
값을 여러 개 꺼내야 하는 코드가 평평해집니다. `else` 블록은 반드시 함수나
반복을 빠져나가야 합니다.

`matches!`는 값이 특정 모양인지 참과 거짓으로만 확인합니다.

nunchi에서 `let ... else`가 가장 많이 쓰이는 자리는 반복 안에서 항목을
건너뛸 때입니다.

# 3.2 `if let`과 `while let`

> **선행 장**: [3.1 `match`](03-1-match.md)
> **연습문제**: 2개

`match`는 모든 경우를 다뤄야 합니다. 그런데 실제로는 **한 경우만 관심 있고
나머지는 아무것도 안 해도 되는 상황**이 많습니다. 그럴 때 쓰는 문법입니다.

## `if let`

```rust
// match 로 쓰면 이렇습니다
match config.rank {
    Some(weights) => apply(weights),
    None => {}                          // 할 일이 없습니다
}

// if let 으로 쓰면 이렇습니다
if let Some(weights) = config.rank {
    apply(weights);
}
```

읽는 방법은 이렇습니다. **"만약 이 모양에 들어맞으면"** 입니다.
`config.rank`가 `Some(...)` 모양이면 안의 값을 `weights`로 꺼내 쓰고,
아니면 아무것도 하지 않습니다.

`else`를 붙일 수도 있습니다.

```rust
if let Some(weights) = config.rank {
    apply(weights);
} else {
    apply(RankWeights::default());
}
```

nunchi 코드에 44번 나옵니다.

```rust
// crates/nunchi-core/src/index.rs 에서
if let Ok(modified) = meta.modified() {
    if let Ok(age) = modified.duration_since(UNIX_EPOCH) {
        file_node.mtime = Some(age.as_secs() as i64);
    }
}
```

파일 수정 시각을 읽습니다. 두 단계 모두 실패할 수 있는데, 실패하면 그냥
`mtime`을 채우지 않으면 됩니다. 오류를 위로 올릴 이유가 없으므로 `?` 대신
`if let`을 씁니다.

> 위 코드는 중첩이 깊습니다. 실제 nunchi 코드는 이 부분을 이터레이터
> 체인으로 고쳐서 한 줄로 만들었습니다. [4.3장](04-3-chains.md)에서 그 형태를
> 보여 드립니다.

## `while let`

같은 판단을 반복에 적용한 것입니다. **모양에 들어맞는 동안 계속 반복합니다.**

```rust
while let Some(item) = queue.pop_front() {
    process(item);
}
```

`pop_front`는 큐가 비면 `None`을 돌려줍니다. 그러면 반복이 끝납니다.

nunchi에 7번 나오며, 대부분 큐를 비울 때까지 도는 경우입니다.

```rust
// crates/nunchi-core/src/store/sqlite.rs 에서
while let Some(path) = queue.pop_front() {
    if path.len() as u32 > max_depth + 1 {
        break;
    }
    let tail = path.last().expect("경로는 비어 있지 않다");
    if tail == to {
        return Ok(vec![path]);
    }
    for next in self.adjacent(tail.as_str(), &[], Direction::Out)? {
        if seen.insert(next.clone()) {
            let mut extended = path.clone();
            extended.push(NodeId(next));
            queue.push_back(extended);
        }
    }
}
```

그래프에서 두 노드 사이의 최단 경로를 찾는 너비 우선 탐색입니다. 큐가 빌
때까지 돌면서 이웃을 계속 넣습니다.

문자열을 훑을 때도 씁니다.

```rust
// crates/nunchi-core/src/framework.rs 에서
while let Some(pos) = lower[from..].find(kw) {
    let start = from + pos;
    // ...
    from = after;
}
```

SQL에서 `FROM`이나 `JOIN` 같은 낱말을 계속 찾습니다. 더 없으면 `find`가
`None`을 돌려주고 반복이 끝납니다.

## `if let`을 언제 쓰지 않는가

**두 경우를 모두 처리해야 하면 `match`가 낫습니다.**

```rust
// 이렇게 쓸 바에는
if let Some(v) = value {
    use_it(v);
} else {
    handle_missing();
}

// match 가 더 읽기 좋습니다
match value {
    Some(v) => use_it(v),
    None => handle_missing(),
}
```

`if let`은 **관심 없는 경우가 있을 때** 쓰는 도구입니다.

**그리고 값을 꺼내 계속 진행해야 하면 `let ... else`가 낫습니다.**
다음 장에서 다룹니다.

## 이 프로젝트에서는

`if let`이 여러 개 겹치면 읽기 어려워집니다. nunchi에도 그런 곳이 있습니다.

```rust
// crates/nunchi-core/src/index.rs 에서
if node.kind == NodeKind::File {
    if let Some(path) = node.path.as_deref() {
        let key = (node.repo.clone(), path.to_string());
        if covered_files.contains(&key) {
            continue;
        }
    }
}
```

세 겹으로 들어가 있습니다. 실제 로직은 "이미 담긴 파일이면 건너뛴다"는 한
문장인데 세 단계를 거칩니다.

`let ... else`를 쓰면 평평해집니다. 다음 장에서 이 코드를 다시 봅니다.

## 연습문제

### 문제 1 [고치기]

```bash
cd book/exercises
cargo test -p ex_03_02_a
```

`match`를 `if let`으로 줄이는 문제입니다.

### 문제 2 [쓰기]

```bash
cargo test -p ex_03_02_b
```

`while let`으로 큐를 비우는 함수를 작성하는 문제입니다.

## 정리

`if let`은 한 경우만 관심 있을 때 씁니다. `match`로 쓰면 아무것도 하지 않는
갈래를 적어야 하는 상황을 줄여 줍니다.

`while let`은 같은 판단을 반복에 적용합니다. 큐가 빌 때까지 도는 경우에
자주 씁니다.

두 경우를 모두 처리해야 하면 `match`가 낫고, 값을 꺼내 계속 진행해야 하면
`let ... else`가 낫습니다.

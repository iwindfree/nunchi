# 1.4 `.clone()`이 170번 나오는 이유

> **선행 장**: [1.3 빌림 `&`와 `&mut`](01-3-borrow.md)
> **연습문제**: 4개

nunchi 코드에 `.clone()`이 170번 나옵니다. 처음 보면 "복사를 이렇게 많이 하면
느리지 않은가"라는 의문이 듭니다. 이 장에서 그 의문에 답합니다.

**결론부터 말하면 일부는 정당하고 일부는 더 나은 설계를 미룬 것입니다.** 둘을 구분할
수 있게 되는 것이 이 장의 목표입니다.

## `Clone`과 `Copy`의 차이

앞 장에서 `Copy`를 다뤘습니다. 둘의 차이는 이렇습니다.

| | `Copy` | `Clone` |
|---|---|---|
| 복사 시점 | 자동입니다 | `.clone()`을 직접 불러야 합니다 |
| 비용 | 저렴합니다 | 비쌀 수 있습니다 |
| 대상 | 스택에만 있는 값 | 힙을 쓰는 값도 가능합니다 |

Rust가 `Clone`을 자동으로 하지 않는 이유가 있습니다. **비용이 드는 일은 코드에
드러나게 하기 때문입니다.** `.clone()`이 코드에 보이면 거기서 힙 할당과 복사가
일어난다는 사실이 드러납니다.

## `.clone()`이 정당한 경우

### 경우 1: 값을 두 곳에 넣어야 할 때

nunchi에서 가장 흔한 경우입니다.

```rust
// crates/nunchi-core/src/index.rs 에서
nodes.push(file_node);
edges.push(Edge::new(
    repo_id.clone(),        // 저장소 노드를 가리키는 엣지를 만듭니다
    file_id.clone(),        // 파일 노드를 가리키는 엣지를 만듭니다
    EdgeKind::Contains,
    Provenance::Fast,
));
table.insert_file(&rel, file_id.clone());
```

`file_id`가 세 곳에 필요합니다. 파일 노드 자체, 엣지, 심볼 표입니다. `Edge`는
`NodeId`를 **소유**해야 하므로 빌려서는 안 됩니다.

```rust
pub struct Edge {
    pub src: NodeId,        // 빌린 것이 아니라 소유합니다
    pub dst: NodeId,
    // ...
}
```

`Edge`가 `&NodeId`를 갖게 만들 수도 있었지만, 그러면 수명 표기가 필요해지고
`Edge`를 함수 밖으로 돌려주기 어려워집니다. 이 문제는
[1.6장](01-6-lifetimes.md)에서 다룹니다.

**그리고 `NodeId`의 복사 비용은 실제로 작습니다.** 안에 든 것이 짧은 문자열
하나이기 때문입니다.

```rust
pub struct NodeId(pub String);   // "file:api/src/OrderService.java" 정도입니다
```

수십 바이트를 복사합니다. 인덱싱 한 번에 수만 번 일어나지만, tree-sitter로
파일을 파싱하는 비용이 그보다 훨씬 큽니다.

### 경우 2: 반복 안에서 같은 값을 계속 써야 할 때

```rust
// crates/nunchi-core/src/index.rs 에서
for route in &fw.routes {
    edges.push(
        Edge::new(
            route_id(&route.method, &route.path),
            NodeId::symbol(repo, &rel, &route.handler),
            EdgeKind::Handles,
            Provenance::Fast,
        )
        .with_confidence(0.9),
    );
}
```

여기서는 `clone`이 없습니다. `route_id`와 `NodeId::symbol`이 매번 새 `NodeId`를
만들기 때문입니다. 복사가 아니라 생성입니다.

반면 아래는 `clone`이 필요합니다.

```rust
// crates/nunchi-core/src/index.rs 에서
for (dst, confidence) in table.resolve_call(&call.callee, &mut stats.calls, &mut tally) {
    if dst == src {
        continue;
    }
    edges.push(
        Edge::new(src.clone(), dst, EdgeKind::Calls, Provenance::Fast)
            .with_confidence(confidence),
    );
}
```

`src`는 반복 밖에서 정해지고 반복마다 필요합니다. 한 호출이 여러 대상으로
해소될 수 있으므로 엣지가 여러 개 생기고, 각 엣지가 `src`를 소유해야 합니다.
그래서 매번 복사합니다.

`dst`에는 `clone`이 없습니다. 반복마다 새로 받은 값이고 그 자리에서 소유권을
넘기면 되기 때문입니다.

### 경우 3: 설정을 여러 곳에서 쓸 때

```rust
// crates/nunchi-cli/src/main.rs 에서
let opts = nunchi_core::pack::PackOptions {
    budget,
    weights: config.rank,
    synonyms: config.semantic.clone(),
    ..Default::default()
};
```

`config.rank`에는 `clone`이 없고 `config.semantic`에는 있습니다. 차이가
무엇입니까?

`RankWeights`는 `f32` 다섯 개이므로 `Copy`입니다. 자동으로 복사됩니다.
`Synonyms`는 안에 `HashMap`이 있으므로 `Copy`가 아니고 `.clone()`이
필요합니다.

이 복사는 팩을 만들 때 한 번만 일어나므로 비용이 문제가 되지 않습니다.

## `.clone()`으로 더 나은 설계를 미룬 경우

솔직히 말하면 nunchi에도 그런 곳이 있습니다.

```rust
// crates/nunchi-core/src/index.rs 에서
let prune_root = root.to_path_buf();
let prune_set = excludes.clone();
let walker = ignore::WalkBuilder::new(root)
    .filter_entry(move |entry| {
        let Some(rel) = npath::relative_to(&prune_root, entry.path()) else {
            return true;
        };
        // ...
    })
    .build();
```

`excludes.clone()`으로 제외 패턴 집합을 통째로 복사합니다. 왜 이렇게 했는지
설명하려면 클로저를 알아야 하므로 [4.1장](04-1-closures.md)에서 다시
다루겠습니다.

짧게 말하면, `filter_entry`에 넘기는 클로저가 `move`로 표시되어 있어서
바깥 값의 소유권을 가져갑니다. 그런데 `excludes`는 함수의 다른 곳에서도
쓰이므로 넘길 수 없습니다. 그래서 복사했습니다.

**다르게 설계할 수 있었습니다.** `Arc`로 감싸서 여러 곳이 같은 데이터를
공유하게 만들면 복사가 사라집니다. `Arc`는 [8.3장](08-3-async.md)에서 다룹니다.
다만 이 복사는 저장소마다 한 번씩만 일어나므로 실제 비용은 무시할 수준입니다.
그래서 고치지 않았습니다.

## 판단 기준

`.clone()`을 만나면 세 가지를 물어보십시오.

**첫째, 얼마나 자주 일어나는가.** 인덱싱 전체에서 한 번이면 무엇을 복사해도
상관없습니다. 반복 안에서 수만 번이면 따져 봐야 합니다.

**둘째, 무엇을 복사하는가.** 짧은 문자열 하나면 수십 바이트입니다. `Vec`이나
`HashMap`이면 안에 든 모든 것이 복사됩니다.

**셋째, 빌림으로 바꿀 수 있는가.** 받은 쪽이 값을 보관해야 하면 빌림으로
바꿀 수 없습니다. 보관하지 않고 읽기만 하면 `&`로 바꿀 수 있습니다.

nunchi의 `clone` 170개는 대부분 첫 번째와 두 번째 기준에서 문제가 없습니다.
`NodeId`처럼 작은 값을 그래프 구조에 넣기 위해 복사하는 경우입니다.

## `.clone()`을 피하는 요령

### 순서를 바꿉니다

```rust
// 이렇게 하면 clone 이 필요합니다
names.push(name.clone());
let msg = format!("added {}", name);

// 순서를 바꾸면 필요 없습니다
let msg = format!("added {}", name);
names.push(name);
```

`format!`은 빌려서 읽기만 하므로 그 뒤에 소유권을 넘길 수 있습니다.
**소유권을 넘기는 동작을 마지막에 두는 것**이 요령입니다.

### 함수가 빌려 받게 만듭니다

```rust
// 소유권을 가져가므로 호출한 쪽에서 clone 이 필요해집니다
fn make_id(path: String) -> String

// 빌리기만 하므로 clone 이 필요 없습니다
fn make_id(path: &str) -> String
```

값을 보관하지 않는 함수는 빌려 받아야 합니다.

### `std::mem::take`를 씁니다

```rust
// crates/nunchi-core/src/framework.rs 에서
if !current.is_empty() {
    parts.push(std::mem::take(&mut current));
}
```

`current`의 내용을 꺼내 가고 그 자리에 빈 문자열을 남깁니다. 복사가 아니라
이동이므로 비용이 없습니다. 반복 안에서 문자열을 계속 만들 때 유용합니다.

## 연습문제

### 문제 1 [읽기]

아래 세 곳의 `clone` 중 없앨 수 있는 것은 무엇입니까?

```rust
// (가)
fn log_and_store(name: String, store: &mut Vec<String>) {
    println!("{}", name.clone());
    store.push(name);
}

// (나)
fn add_two_edges(id: NodeId, edges: &mut Vec<(NodeId, NodeId)>) {
    edges.push((id.clone(), NodeId("a".into())));
    edges.push((id, NodeId("b".into())));
}

// (다)
fn describe(config: &Config) -> String {
    let name = config.solution.name.clone();
    format!("solution: {}", name)
}
```

<details>
<summary>정답 보기</summary>

(가)와 (다)에서 없앨 수 있습니다.

(가)에서 `println!`은 값을 빌려서 읽기만 하므로 `clone`이 필요 없습니다.
`println!("{}", name);`으로 충분합니다.

(나)의 `clone`은 필요합니다. 엣지 두 개가 각각 `NodeId`를 소유해야 하는데,
첫 번째에 넘기면 두 번째에 쓸 것이 없어집니다. 두 번째에는 `clone`이 없는데,
그것이 맞습니다. 마지막 사용에서는 소유권을 넘기면 됩니다.

(다)에서도 필요 없습니다. `format!`이 빌려서 읽으므로
`format!("solution: {}", config.solution.name)`으로 충분합니다.

</details>

### 문제 2 [고치기]

```bash
cd book/exercises
cargo test -p ex_01_04_a
```

불필요한 `clone`을 없애는 문제입니다.

### 문제 3 [고치기]

```bash
cargo test -p ex_01_04_b
```

`clone` 없이 두 곳에 값을 넣어야 하는 문제입니다. 함수 서명을 바꿔야 합니다.

### 문제 4 [읽기]

`NodeId`의 `clone` 비용과 `Vec<Node>`의 `clone` 비용을 비교하십시오.

<details>
<summary>정답 보기</summary>

`NodeId`는 `String` 하나를 감싼 것이므로 힙 할당 한 번과 수십 바이트 복사가
일어납니다.

`Vec<Node>`는 목록 안의 모든 `Node`를 복사합니다. `Node`에는 `String`이
여러 개(`name`, `signature`, `doc` 등) 들어 있으므로, 노드가 1,000개라면
힙 할당이 수천 번 일어납니다.

이 차이 때문에 nunchi 코드에서 `NodeId`는 자유롭게 복사하지만 `Vec<Node>`는
언제나 `&[Node]`로 빌려서 넘깁니다.

```rust
pub fn upsert_nodes(&mut self, nodes: &[Node]) -> Result<usize>
//                                    ^ 빌립니다
```

</details>

## 정리

`Clone`은 자동이 아니라 직접 불러야 합니다. Rust가 비용이 드는 일을 코드에
드러나게 하기 때문입니다.

nunchi의 `clone` 170개는 대부분 정당합니다. `NodeId`처럼 작은 값을 그래프
구조에 넣기 위해 복사하는 경우이며, 받는 쪽이 값을 소유해야 하므로 빌림으로
바꿀 수 없습니다.

`clone`을 만나면 얼마나 자주 일어나는지, 무엇을 복사하는지, 빌림으로 바꿀 수
있는지를 물어보십시오. 피하는 요령은 소유권을 넘기는 동작을 마지막에 두고,
값을 보관하지 않는 함수는 빌려 받게 만드는 것입니다.

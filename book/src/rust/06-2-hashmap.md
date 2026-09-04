# 6.2 `HashMap`과 `HashSet`

> **선행 장**: [6.1 `Vec<T>`와 슬라이스](06-1-vec.md)
> **연습문제**: 2개

키로 값을 찾는 표와, 값이 있는지만 보는 집합입니다. nunchi에 55번 나옵니다.

## `HashMap<K, V>`

```rust
use std::collections::HashMap;

let mut by_name: HashMap<String, Vec<NodeId>> = HashMap::new();
by_name.insert("findOne".to_string(), vec![id]);

if let Some(ids) = by_name.get("findOne") {
    println!("{}개", ids.len());
}
```

자주 쓰는 메서드입니다.

| 메서드 | 하는 일 |
|---|---|
| `.insert(k, v)` | 넣습니다. 이미 있으면 덮어씁니다 |
| `.get(&k)` | 빌려서 봅니다. 없으면 `None`입니다 |
| `.get_mut(&k)` | 빌려서 바꿉니다 |
| `.contains_key(&k)` | 있는지만 봅니다 |
| `.remove(&k)` | 지우면서 값을 돌려줍니다 |
| `.entry(k)` | 없으면 만들고 있으면 가져옵니다 |
| `.len()`, `.is_empty()` | 개수를 봅니다 |
| `.values()`, `.keys()` | 값이나 키만 훑습니다 |

## `.entry()`가 핵심입니다

"없으면 만들고 있으면 가져온다"를 한 번에 합니다.

```rust
// 이렇게 쓰면 두 번 찾습니다
if !map.contains_key(&name) {
    map.insert(name.clone(), Vec::new());
}
map.get_mut(&name).unwrap().push(id);

// entry 를 쓰면 한 번입니다
map.entry(name).or_default().push(id);
```

nunchi에서 자주 쓰입니다.

```rust
// crates/nunchi-core/src/resolve.rs 에서
pub fn insert_symbol(&mut self, name: &str, id: NodeId, kind: &str) {
    self.kinds.insert(id.0.clone(), kind.to_string());
    self.by_name.entry(name.to_string()).or_default().push(id);
}
```

같은 이름의 심볼이 여러 개일 수 있으므로 값이 `Vec<NodeId>`입니다.
`or_default()`가 없으면 빈 목록을 만들어 넣고, 있으면 그대로 가져옵니다.
그다음 `push`로 새 심볼을 더합니다.

`or_insert_with`도 있습니다. 기본값을 직접 만들 때 씁니다.

```rust
// crates/nunchi-core/src/history.rs 에서
let author_id = authors
    .entry(author_email.to_string())
    .or_insert_with(|| {
        let id = NodeId(format!("author:{author_email}"));
        let mut n = Node::new(id.clone(), NodeKind::Author, author_name, repo);
        n.attrs = serde_json::json!({ "email": author_email });
        nodes.push(n);
        id
    })
    .clone();
```

저자가 처음 나오면 노드를 만들어 넣고, 이미 있으면 그 값을 씁니다.
**클로저 안에서 `nodes`에 노드를 넣는 부수 효과가 있습니다.** 저자가
처음 나올 때만 노드를 만들기 위해서입니다.

## `HashSet<T>`

값이 있는지만 보는 집합입니다. 중복이 자동으로 걸러집니다.

```rust
use std::collections::HashSet;

let mut seen: HashSet<String> = HashSet::new();
if seen.insert(name.clone()) {
    // 처음 보는 이름입니다
}
```

`.insert()`가 참과 거짓을 돌려주는 점이 중요합니다. **처음 넣으면 참,
이미 있으면 거짓입니다.** 이것으로 중복 검사와 삽입을 한 번에 합니다.

```rust
// crates/nunchi-core/src/store/sqlite.rs 에서
for next in self.adjacent(tail.as_str(), &[], Direction::Out)? {
    if seen.insert(next.clone()) {
        let mut extended = path.clone();
        extended.push(NodeId(next));
        queue.push_back(extended);
    }
}
```

그래프를 훑으면서 이미 방문한 노드를 다시 큐에 넣지 않습니다. `insert`가
거짓을 돌려주면 이미 본 노드입니다.

## 키가 되려면 조건이 있습니다

`HashMap`의 키와 `HashSet`의 값은 `Hash`와 `Eq`를 갖고 있어야 합니다.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(pub String);
```

`NodeId`에 `Hash`와 `Eq`가 붙어 있으므로 키로 쓸 수 있습니다.

**실수는 `HashMap`의 키로 쓸 수 없습니다.** `f32`와 `f64`에는 `Eq`가
없기 때문입니다. `NaN`은 자기 자신과도 같지 않아서 완전한 동등 비교가
성립하지 않습니다.

## 순서가 없습니다

`HashMap`과 `HashSet`을 훑으면 순서가 매번 달라집니다. 순서가 필요하면
`BTreeMap`이나 `BTreeSet`을 씁니다. 키 순서대로 정렬되어 있습니다.

```rust
// crates/nunchi-core/src/index.rs 에서
pub by_lang: BTreeMap<String, (usize, usize)>,
```

언어별 통계를 담습니다. `doctor`가 출력할 때 순서가 매번 바뀌면 읽기
어려우므로 `BTreeMap`을 썼습니다.

## 튜플을 키로 씁니다

```rust
// crates/nunchi-core/src/history.rs 에서
let mut pair_counts: HashMap<(String, String), usize> = HashMap::new();
// ...
*pair_counts.entry((a, b)).or_default() += 1;
```

파일 두 개가 함께 바뀐 횟수를 셉니다. 키가 `(파일1, 파일2)` 짝입니다.

`*`가 앞에 붙은 이유는 `entry(...).or_default()`가 값을 **빌려서** 돌려주기
때문입니다. 빌린 것에 값을 더하려면 참조를 해제해야 합니다.

## 연습문제

### 문제 1 [고치기]

```bash
cd book/exercises
cargo test -p ex_06_02_a
```

`contains_key`와 `insert`를 `entry`로 줄이는 문제입니다.

### 문제 2 [쓰기]

```bash
cargo test -p ex_06_02_b
```

`HashSet`으로 중복을 거르는 문제입니다.

## 정리

`HashMap`은 키로 값을 찾고 `HashSet`은 값이 있는지만 봅니다. 키에는 `Hash`와
`Eq`가 필요하며, 실수는 키가 될 수 없습니다.

`.entry()`는 "없으면 만들고 있으면 가져온다"를 한 번에 하며, 표를 두 번 찾지
않습니다.

`HashSet::insert`는 처음 넣으면 참을 돌려주므로 중복 검사와 삽입을 동시에
할 수 있습니다.

순서가 필요하면 `BTreeMap`과 `BTreeSet`을 씁니다.

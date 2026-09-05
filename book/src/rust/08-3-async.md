# 8.3 `Arc`와 `async`

> **선행 장**: [1.2 이동과 복사](01-2-move.md), [5.2 트레이트](05-2-traits.md)
> **연습문제**: 1개

MCP 서버에만 나오는 두 가지입니다. nunchi 전체에서 `async fn`이 두 개,
`Arc`가 두 개뿐이므로 필요한 만큼만 다룹니다.

## `Arc`는 여러 곳이 같은 값을 공유하게 합니다

[1.1장](01-1-ownership.md)에서 "소유자는 하나"라고 했습니다. 그런데 여러 곳이
같은 값을 봐야 하고, 그중 누가 마지막까지 살아 있을지 미리 알 수 없는 경우가
있습니다.

`Arc`는 **소유자를 여럿으로 만듭니다.** 대신 몇 곳에서 갖고 있는지 세어 두고,
0이 되면 값을 없앱니다.

```rust
use std::sync::Arc;

let shared = Arc::new(big_data);
let a = Arc::clone(&shared);      // 소유자가 둘이 됩니다
let b = Arc::clone(&shared);      // 셋이 됩니다
```

`Arc::clone`은 **데이터를 복사하지 않습니다.** 참조 횟수만 하나 올립니다.
그래서 큰 데이터도 비용 없이 공유할 수 있습니다.

`Arc`는 atomically reference counted의 줄임말입니다. 여러 스레드가 동시에
세어도 안전합니다. 한 스레드에서만 쓸 것이 확실하면 `Rc`가 더 빠르지만,
nunchi에는 없습니다.

nunchi에서 `Arc`가 나오는 자리는 MCP 도구 정의입니다.

```rust
// crates/nunchi-cli/src/serve.rs 에서
fn schema(props: serde_json::Value, required: &[&str]) -> Arc<JsonObject> {
    let value = serde_json::json!({
        "type": "object",
        "properties": props,
        "required": required,
    });
    let obj = value.as_object().cloned().unwrap_or_default();
    Arc::new(obj)
}
```

`rmcp` 라이브러리가 `Arc<JsonObject>`를 요구합니다. 도구 정의를 여러 곳에서
읽어야 하고 언제 없어질지 라이브러리가 정하기 때문입니다.

## `async`는 기다리는 동안 다른 일을 하게 합니다

MCP 서버는 표준 입력에서 요청을 읽고 표준 출력으로 답을 보냅니다.
**읽는 동안에는 아무 일도 하지 않고 기다립니다.**

보통 함수라면 그 스레드가 통째로 멈춥니다. `async` 함수는 기다리는 동안
스레드를 다른 작업에 양보합니다.

```rust
// crates/nunchi-cli/src/serve.rs 에서
async fn call_tool(
    &self,
    request: CallToolRequestParams,
    _context: RequestContext<RoleServer>,
) -> Result<CallToolResponse, McpError> {
    // ...
}
```

`async fn`은 부른다고 바로 실행되지 않습니다. **실행할 준비가 된 것을
돌려줍니다.** 그것을 실제로 실행하려면 실행기가 필요합니다.

```rust
// crates/nunchi-cli/src/serve.rs 에서
pub fn run(config: Config, db_path: PathBuf) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(async move {
        let service = NunchiServer::new(config, db_path)
            .serve(rmcp::transport::io::stdio())
            .await?;
        service.waiting().await?;
        Ok::<_, anyhow::Error>(())
    })
}
```

`tokio`가 그 실행기입니다. `block_on`이 "이 비동기 작업이 끝날 때까지 여기서
기다린다"는 뜻이고, `.await`가 "이 작업이 끝날 때까지 양보한다"는 뜻입니다.

## nunchi에서 `async`가 두 개뿐인 이유

MCP 프로토콜을 다루는 `rmcp` 라이브러리가 비동기로 되어 있어서 어쩔 수 없이
따라간 것입니다.

**나머지 코드는 전부 보통 함수입니다.** 인덱싱과 팩 만들기는 계산이 대부분이고
기다리는 시간이 거의 없으므로 비동기로 만들 이유가 없습니다. 비동기는 코드를
복잡하게 만들므로 필요할 때만 씁니다.

MCP 서버 안에서도 실제 작업은 보통 함수를 부릅니다.

```rust
"nunchi_pack" => {
    let task = arg_str(&request, "task")?;
    let budget = arg_usize(&request, "budget", 4000);
    let graph = MemGraph::load(&store).map_err(err)?;      // 보통 함수입니다
    let result = pack::build_pack(&store, &graph, &task, &self.roots, &opts)
        .map_err(err)?;                                     // 보통 함수입니다
    Ok(ok_json(serde_json::to_value(result).unwrap_or_default()))
}
```

**비동기는 바깥 계층에만 있고 안쪽은 보통 코드입니다.** 이 구조를 유지하면
비동기가 코드 전체로 번지지 않습니다.

## 로그를 표준 오류로 보내야 하는 이유

MCP 서버에서 실제로 겪은 문제입니다.

```rust
// crates/nunchi-cli/src/main.rs 에서
tracing_subscriber::fmt()
    .with_writer(std::io::stderr)     // 이것이 없으면 프로토콜이 깨집니다
    .init();
```

`tracing_subscriber`는 기본으로 표준 출력에 씁니다. 그런데 stdio를 쓰는 MCP
서버에서 **표준 출력은 JSON-RPC 메시지 전용입니다.** 로그가 섞이면 클라이언트가
응답을 파싱하지 못합니다.

처음에 이것을 몰라서 `initialize`는 성공하는데 그다음 요청이 실패했습니다.
표준 오류로 보내도록 고쳤습니다.

## 연습문제

### 문제 1 [읽기]

`Arc::clone`이 데이터를 복사하지 않는데도 `clone`이라는 이름을 쓰는 이유는
무엇입니까?

<details>
<summary>정답 보기</summary>

`Clone` 트레이트를 구현했기 때문입니다.

`Arc<T>`에 `.clone()`을 부르면 **`Arc` 자체가 복사됩니다.** 안에 든
데이터가 아니라 "가리키는 정보와 참조 횟수"가 복사됩니다.

`Arc::clone(&shared)`와 `shared.clone()`은 같은 일을 합니다. 앞의 형태를 쓰는
이유는 읽는 사람에게 **데이터 복사가 아니라 참조 복사임을 분명히 하기
위해서**입니다. 관례입니다.

</details>

### 문제 2 [쓰기]

```bash
cd book/exercises
cargo test -p ex_08_03_a
```

`Arc`로 값을 공유하는 문제입니다.

## 정리

`Arc`는 여러 곳이 같은 값을 공유하게 하며, 몇 곳에서 갖고 있는지 세어 두었다가
0이 되면 없앱니다. `Arc::clone`은 데이터를 복사하지 않고 숫자만 올립니다.

`async` 함수는 기다리는 동안 스레드를 다른 작업에 양보합니다. 실행하려면 `tokio` 같은
실행기가 필요합니다.

nunchi에서 비동기는 MCP 서버의 바깥 계층에만 있고 안쪽은 보통 함수입니다. 이
구조를 유지하면 비동기가 코드 전체로 번지지 않습니다.

stdio를 쓰는 MCP 서버에서 표준 출력은 JSON-RPC 전용이므로 로그는 반드시
표준 오류로 보내야 합니다.

# 8.2 채널 `mpsc`로 워처 만들기

> **선행 장**: [1.2 이동과 복사](01-2-move.md), [4.1 클로저](04-1-closures.md)
> **연습문제**: 1개

파일 워처가 다른 스레드에서 알림을 보냅니다. 그 값을 받는 방법입니다.

## 문제 상황

`notify` 라이브러리는 파일이 바뀔 때마다 클로저를 부릅니다. **그런데 그
클로저는 우리 스레드가 아니라 라이브러리가 만든 스레드에서 호출됩니다.**

```
[워처 스레드]                    [본체 스레드]
파일 변경 감지                    무엇을 해야 하나?
  → 클로저 실행                   
```

클로저 안에서 인덱싱을 바로 하면 안 됩니다. 파일 하나 바뀔 때마다 인덱싱이
실행되고, 여러 스레드가 동시에 데이터베이스를 건드리게 됩니다.

**변경 사실만 본체 스레드로 보내야 합니다.** 그 통로가 채널입니다.

## 채널은 한 방향 통로입니다

`mpsc`는 multi-producer, single-consumer의 줄임말입니다. 보내는 쪽이 여럿일
수 있고 받는 쪽은 하나입니다.

```rust
use std::sync::mpsc;

let (tx, rx) = mpsc::channel::<Event>();
```

`tx`는 보내는 쪽(transmitter), `rx`는 받는 쪽(receiver)입니다.

```rust
tx.send(event);              // 보냅니다
let event = rx.recv();       // 받습니다. 올 때까지 기다립니다
```

## 이 프로젝트에서는

```rust
// crates/nunchi-cli/src/watch.rs 에서
let (tx, rx) = mpsc::channel::<Event>();
let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
    if let Ok(event) = res {
        let _ = tx.send(event);
    }
})
.context("파일 워처를 만들 수 없습니다")?;
```

세 가지를 짚습니다.

**첫째, `move`가 필수입니다.** 클로저가 다른 스레드에서 호출되므로 `tx`를
빌려서 쓸 수 없습니다. 소유권을 가져가야 합니다([4.1장](04-1-closures.md)).

**둘째, `let _ =`로 실패를 무시합니다.** 받는 쪽이 이미 사라졌으면 `send`가
실패하는데, 그때는 프로그램이 끝나는 중이므로 신경 쓸 이유가 없습니다
([2.2장](02-2-result.md)).

**셋째, 클로저 안에서 아무 일도 하지 않습니다.** 사건을 그대로 보내기만
합니다. 판단은 본체 스레드가 합니다.

## 받는 쪽

```rust
// crates/nunchi-cli/src/watch.rs 에서
loop {
    match rx.recv_timeout(Duration::from_millis(200)) {
        Ok(event) => {
            // 변경 파일 목록에 모읍니다
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {}
        Err(mpsc::RecvTimeoutError::Disconnected) => break,
    }
    // debounce 시간이 지났는지 확인하고 인덱싱합니다
}
```

`recv()`가 아니라 `recv_timeout()`을 씁니다. 이유가 있습니다.

`recv()`는 사건이 올 때까지 **무한정 기다립니다.** 그러면 debounce 시간이
지났는지 확인할 기회가 없습니다. 200밀리초마다 깨어나서 확인해야 합니다.

세 가지 경우를 나눕니다.

- `Ok(event)`는 사건이 왔습니다.
- `Timeout`은 200밀리초 동안 아무 일도 없었습니다. 정상이므로 아무것도 하지
  않습니다.
- `Disconnected`는 보내는 쪽이 전부 사라졌습니다. 워처가 종료되었다는 뜻이므로
  반복을 끝냅니다.

## debounce가 필요한 이유

`git checkout`을 하면 파일 수천 개가 한꺼번에 바뀝니다. 사건마다 인덱싱하면
수천 번 실행하게 됩니다.

```rust
// crates/nunchi-cli/src/watch.rs 에서
const DEBOUNCE: Duration = Duration::from_millis(500);

// ...
if !pending.is_empty() {
    last_event = Some(Instant::now());
}

// 변경이 멎고 500밀리초가 지나야 실제로 인덱싱합니다
let ready = last_event.is_some_and(|t| t.elapsed() >= DEBOUNCE);
if !ready || pending.is_empty() {
    continue;
}
```

**사건이 올 때마다 타이머를 다시 시작합니다.** 변경이 계속되는 동안에는
인덱싱하지 않고, 500밀리초 동안 조용하면 그때 한 번에 처리합니다.

## 무한 반복을 막습니다

```rust
for p in event.paths {
    if p.components().any(|c| c.as_os_str() == ".nunchi" || c.as_os_str() == ".git")
    {
        continue;
    }
    pending.insert(p);
}
```

인덱싱하면 `.nunchi/graph.db`가 바뀝니다. 그것이 사건이 되어 또 인덱싱하면
멈추지 않습니다. 그래서 인덱스 디렉터리와 `.git`의 변경은 무시합니다.

## 다른 방법과 비교하면

여러 스레드가 값을 공유하는 방법이 몇 가지 있습니다.

| 방법 | 언제 |
|---|---|
| 채널 | 한 방향으로 값을 보낼 때 |
| `Arc<Mutex<T>>` | 여러 스레드가 같은 값을 읽고 쓸 때 |
| `Arc<T>` | 여러 스레드가 읽기만 할 때 |

**nunchi는 채널만 씁니다.** 워처가 본체에게 알리기만 하면 되고, 반대 방향은
필요 없기 때문입니다. `Mutex`가 없으면 교착 상태를 걱정할 일도 없습니다.

## 연습문제

### 문제 1 [쓰기]

```bash
cd book/exercises
cargo test -p ex_08_02_a
```

채널로 값을 보내고 모으는 문제입니다.

## 정리

채널은 스레드 사이에 한 방향으로 값을 보내는 통로입니다. `mpsc::channel()`이
보내는 쪽과 받는 쪽을 함께 돌려줍니다.

다른 스레드에서 호출되는 클로저는 `move`로 소유권을 가져가야 합니다.

`recv()`는 무한정 기다리므로, 주기적으로 다른 일을 해야 하면
`recv_timeout()`을 씁니다.

nunchi의 워처는 사건을 모으기만 하고 500밀리초 동안 조용해진 뒤에 인덱싱합니다.
브랜치를 전환하면 파일 수천 개가 한꺼번에 바뀌기 때문입니다.

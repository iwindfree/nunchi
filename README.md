# nunchi

코드베이스 컨텍스트 그래프. 이름은 한국어 **눈치**(말해지지 않은 맥락을 읽는 능력)에서 왔다.

에이전트가 파일을 대량으로 읽어 구조를 재구성하는 대신, 미리 계산된 그래프에 질의해
**압축된 사실 + 정확한 좌표(`path:line`)** 를 받게 한다.

설계 문서: [PLAN.md](PLAN.md)

## 빠른 시작

```bash
cargo build --release

./target/release/nunchi init ~/dev/order-api ~/dev/order-web --name web
./target/release/nunchi index
./target/release/nunchi doctor
./target/release/nunchi find "주문 재시도"
```

`nunchi init`이 만든 `nunchi.toml`의 **제외 패턴을 반드시 확인할 것.**
생성 코드나 벤더 디렉터리가 인덱스에 들어오면 랭킹이 오염된다.

## 명령

| 명령 | 상태 |
|---|---|
| `nunchi init` | ✅ 저장소·언어 감지, `nunchi.toml` 생성 |
| `nunchi index [--rebuild]` | ✅ 파일 워크 + 내용 해시 (File 노드까지) |
| `nunchi doctor [--json]` | ✅ 언어 커버리지, 노드/엣지 수 |
| `nunchi find <query> [--json]` | ✅ FTS5 전문 검색 |
| `nunchi serve` | ⬜ Phase 1 — MCP 서버 (rmcp) |
| `nunchi pack <task>` | ⬜ Phase 2 — 토큰 예산 컨텍스트 팩 |
| `nunchi tui` | ⬜ Phase 3.5 — ratatui |

## 구조

```
crates/
├── nunchi-core/          라이브러리
│   ├── model.rs          노드 18종 / 엣지 19종
│   ├── path.rs           경로 정규화 (Windows 대응)
│   ├── lang.rs           언어 판별
│   ├── config.rs         nunchi.toml
│   ├── index.rs          파일 워크 + 해시
│   └── store/
│       ├── mod.rs        Store 트레이트 — 6개 메서드
│       └── sqlite.rs     SQLite(WAL) + FTS5 구현
└── nunchi-cli/           단일 바이너리 `nunchi`
```

**`Store` 트레이트를 좁게 유지하는 것이 설계의 핵심이다.** v1은 SQLite로 가지만
엔진 스파이크 결과에 따라 LadybugDB 등으로 갈아탈 수 있어야 하며,
그 교체 비용을 하루 이내로 묶는 장치가 그 6개 메서드다. (PLAN.md 2절)

## 개발

```bash
cargo test
cargo build --release
```

macOS와 Windows 양쪽에서 네이티브 빌드한다(크로스 컴파일 없음, PLAN.md 3.10절).

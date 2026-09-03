# nunchi

코드베이스 컨텍스트 그래프. 이름은 한국어 **눈치**(말해지지 않은 맥락을 읽는 능력)에서 왔다.

에이전트가 파일을 대량으로 읽어 구조를 재구성하는 대신, 미리 계산된 그래프에 질의해
**압축된 사실 + 정확한 좌표(`path:line`)** 를 받게 한다.

- **[사용법](docs/USAGE.md)** — 설치, 온보딩, 명령, 확장
- **[원리](docs/PRINCIPLES.md)** — 어떻게 토큰을 줄이는가
- **[개발 문서](docs/DEVELOPMENT.md)** — 코드를 고치려는 사람을 위해
- **[설계 문서](PLAN.md)** — 결정의 배경과 대안 검토

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
| `nunchi index [--rebuild] [--watch]` | ✅ tree-sitter 추출 + 참조 해소 + git 이력 |
| `nunchi doctor [--json]` | ✅ 커버리지 · 연결률 · 교차 저장소 연결 |
| `nunchi find <query> [--json]` | ✅ FTS5 전문 검색 |
| `nunchi pack <task> [--json]` | ✅ 토큰 예산 컨텍스트 팩 — 가장 많이 쓰는 명령 |
| `nunchi serve` | ✅ MCP 서버 (rmcp 3.2) — 툴 5개 |
| `nunchi tui` | ✅ 탐색·영향범위·인덱스·팩 미리보기·지표 |
| `nunchi rules [--toml]` | ✅ 프레임워크 규칙 확인·복사 |
| `nunchi bench [--tasks f]` | ✅ grounded vs ungrounded 토큰·recall 실측 |
| `nunchi index --watch` | ✅ 파일 워처 + 증분 재인덱싱 |

## 구조

```
crates/
├── nunchi-core/          라이브러리 — 모든 로직
│   ├── model.rs          노드 18종 / 엣지 19종
│   ├── extract.rs        tree-sitter 심볼 추출 (5개 언어)
│   ├── framework.rs      Spring·FastAPI·Flask·ASP.NET·React·JPA·MyBatis
│   ├── mapper_xml.rs     MyBatis XML 매퍼
│   ├── history.rs        git 이력 → 동시변경
│   ├── bench.rs          벤치 하네스
│   ├── resolve.rs        이름 기반 참조 해소
│   ├── graph.rs          메모리 그래프 + PPR
│   ├── pack.rs           랭킹 + 토큰 예산 렌더링
│   ├── cache.rs          콘텐츠 주소 추출 캐시
│   ├── rules.rs          프레임워크 규칙 (설정 데이터)
│   └── store/            Store 트레이트 + SQLite 구현
└── nunchi-cli/           단일 바이너리 — main / serve / watch / tui
```

전체 구조는 [개발 문서](docs/DEVELOPMENT.md)를 보세요.

**`Store` 트레이트를 좁게 유지하는 것이 설계의 핵심이다.** v1은 SQLite로 가지만
엔진 스파이크 결과에 따라 LadybugDB 등으로 갈아탈 수 있어야 하며,
그 교체 비용을 하루 이내로 묶는 장치가 그 6개 메서드다. (PLAN.md 2절)

## 개발

```bash
cargo test
cargo build --release
```

macOS와 Windows 양쪽에서 네이티브 빌드한다(크로스 컴파일 없음, PLAN.md 3.10절).

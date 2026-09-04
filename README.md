# nunchi

코드베이스 컨텍스트 그래프입니다. 이름은 한국어 낱말인 **눈치**에서 따왔습니다.
말해지지 않은 맥락을 읽어내는 능력을 뜻하는데, 이 도구가 코딩 에이전트에게
제공하려는 것이 바로 그것입니다.

에이전트가 파일을 대량으로 읽어서 코드 구조를 매번 다시 파악하는 대신,
미리 계산해 둔 그래프에 질의하여 **압축된 사실과 정확한 좌표**(`path:line`)를
받도록 만듭니다.

## 빠른 시작

```bash
cargo build --release

./target/release/nunchi init ~/dev/order-api ~/dev/order-web --name web
./target/release/nunchi index
./target/release/nunchi doctor
./target/release/nunchi pack "주문 재시도 로직 수정"
```

Rust 1.90 이상이 필요합니다. macOS와 Windows에서 각각 네이티브로 빌드하며,
크로스 컴파일은 사용하지 않습니다.

## 문서

| 문서 | 읽어야 하는 사람 |
|---|---|
| [사용 안내서](docs/GUIDE.md) | 이 도구를 실제로 쓰려는 사람 |
| [설계 문서](docs/DESIGN.md) | 왜 토큰이 줄어드는지, 왜 이렇게 만들었는지 알고 싶은 사람 |
| [기여 안내서](docs/CONTRIBUTING.md) | 코드를 고치거나 이어받으려는 사람 |
| [학습용 책](book/src/intro.md) | Rust와 이 코드를 처음부터 배우려는 사람 |

처음 접하셨다면 설계 문서를 먼저 읽으시기를 권합니다. 이 도구가 무엇을 하는지
이해하지 못한 상태에서는 사용 안내서의 각 단계가 왜 필요한지 알기 어렵습니다.

## 명령

| 명령 | 하는 일 |
|---|---|
| `nunchi init` | 설정 파일을 만들고 저장소와 언어를 자동으로 감지합니다 |
| `nunchi index [--rebuild] [--watch]` | 인덱스를 만들거나 갱신합니다 |
| `nunchi doctor [--json]` | 인덱스 품질을 검증합니다 |
| `nunchi pack <task> [--json]` | 컨텍스트 팩을 만듭니다. 가장 자주 쓰는 명령입니다 |
| `nunchi find <query> [--json]` | 심볼과 파일을 검색합니다 |
| `nunchi serve` | MCP 서버를 실행합니다 |
| `nunchi bench [--tasks f]` | 토큰 절감량과 재현율을 측정합니다 |
| `nunchi rules [--toml]` | 적용 중인 프레임워크 규칙을 출력합니다 |
| `nunchi tui` | 대화형 화면에서 그래프를 탐색하고 가중치를 조정합니다 |

## 지원 범위

| 언어 | 심볼 | 라우트 | 의존성 주입 | 영속 계층 |
|---|---|---|---|---|
| Java | 지원 | Spring | 지원 | JPA, MyBatis(어노테이션과 XML) |
| TypeScript, JavaScript | 지원 | 일부 | 없음 | 없음 |
| Python | 지원 | FastAPI, Flask | 없음 | SQLAlchemy |
| C# | 지원 (partial 병합) | ASP.NET | 없음 | 없음 |
| Rust | 지원 | 없음 | 없음 | 없음 |

프레임워크 지원은 설정 파일에 규칙을 추가하여 넓힐 수 있습니다. 바이너리를
다시 빌드할 필요가 없습니다. 자세한 방법은 [사용 안내서](docs/GUIDE.md)에
정리해 두었습니다.

## 개발

```bash
cargo test
cargo build --release
```

크레이트 구조와 작업 절차는 [기여 안내서](docs/CONTRIBUTING.md)를 참고하시기 바랍니다.

## 학습용 책

Rust 문법 28장과 nunchi 코드 설명 12장, 연습문제 57개로 이루어진 책이
`book/`에 있습니다. Rust를 처음 접하는 사람도 읽을 수 있게 썼습니다.

```bash
cargo install mdbook
cd book && mdbook serve --open
```

연습문제는 이렇게 풉니다.

```bash
cd book/exercises
cargo test -p ex_01_04_a
```

# 사용법

## 설치

```bash
git clone https://github.com/iwindfree/nunchi.git
cd nunchi
cargo build --release
# 산출물: target/release/nunchi (Windows는 nunchi.exe)
```

Rust 1.90 이상이 필요합니다. macOS와 Windows 양쪽에서 각각 네이티브 빌드하며,
크로스 컴파일하지 않습니다.

`nunchi`를 PATH에 두면 편합니다:

```bash
cp target/release/nunchi ~/.local/bin/     # macOS/Linux
```

---

## 4단계 온보딩

```bash
nunchi init ~/dev/order-api ~/dev/order-web --name web
nunchi index
nunchi doctor
nunchi serve      # MCP 등록 후 에이전트가 사용
```

### ① `nunchi init`

저장소를 훑어 언어를 감지하고 `nunchi.toml`을 만듭니다.
여러 저장소를 하나의 **솔루션**으로 묶으면 그 사이에 교차 엣지가 생깁니다.

```toml
[solution]
name = "web"
repos = ["/Users/me/dev/order-api", "/Users/me/dev/order-web"]

[index]
languages = ["java", "typescript"]     # java · typescript · javascript · python · csharp · rust
exclude = ["**/node_modules/**", "**/build/**", "**/target/**", ...]
max_file_bytes = 2097152
max_commits = 1000          # git 이력을 읽을 커밋 수. 0이면 생략

```

`init`은 파일을 **두 개** 만듭니다.

| 파일 | 내용 | 커밋? |
|---|---|---|
| `nunchi.toml` | 저장소 **절대 경로** — 머신마다 다름 | ❌ gitignore |
| `nunchi.shared.toml` | 랭킹 가중치 · 프레임워크 규칙 · 용어 사전 · 제외 패턴 | ✅ **커밋** |

공용 파일이 머신 로컬 값을 덮어씁니다. 회사 장비와 개인 장비가 **같은 가중치와
같은 규칙**을 쓰게 하려면 `nunchi.shared.toml`을 커밋하세요.

```toml
# nunchi.shared.toml — 경로가 들어가지 않는다
[rank]
alpha_bm25 = 0.7
beta_ppr = 0.5
gamma_recency = 0.3      # 반감기 30일 지수 감쇠
delta_cochange = 0.4
epsilon_central = 0.2
```

> **제외 패턴을 반드시 확인하세요.** 생성 코드나 벤더 디렉터리가 인덱스에 들어오면
> 랭킹이 오염되고 팩이 쓰레기를 반환합니다. 온보딩 품질의 절반이 여기서 갈립니다.

### ② `nunchi index`

```bash
nunchi index                # 증분 (콘텐츠 주소 캐시 사용)
nunchi index --rebuild      # 인덱스를 지우고 처음부터
nunchi index --watch        # 데몬. 파일 변경을 감시하며 재인덱싱
```

`--watch`는 500ms debounce로 변경을 묶습니다. 브랜치를 전환해도 이벤트 폭풍이
나지 않고, 되돌아올 때는 캐시 적중으로 재파싱이 0회가 됩니다.

인덱싱이 끝나면 **이번에 발견되지 않은 파일의 노드가 자동으로 제거**되고,
이어서 아무도 참조하지 않게 된 의존성·커밋·저자 노드도 정리됩니다.
삭제·이동한 파일 때문에 `--rebuild`를 돌릴 필요가 없습니다.

### ③ `nunchi doctor` — 진짜 관문

```bash
nunchi doctor
nunchi doctor --json        # CI 게이트용
```

첫 인덱싱은 대개 "돌긴 도는데 품질이 나쁜" 상태로 끝납니다. 이 명령이 그걸 드러냅니다.

```
언어 커버리지
  java          80 files   80 파싱  100.0%  ✓
  javascript    50 files   50 파싱  100.0%  ✓
· sql           15 files       — 파서 없음

호출 연결률                     23.4%
  호출 1606 — 해소 199 · 모호 94 · 미해소 1161 · 후보과다 152
  미해소 호출 상위 — 외부 API면 정상, 내부 심볼이면 추출기 결함
    assertThat    79      ← AssertJ. 정상
    save          45      ← JPA 리포지터리. 본문이 없음 (알려진 한계)
    builder       39      ← Lombok 생성 코드. 정상

프레임워크 의미론
  라우트 19 · Bean 32 · 주입 48해소/15미해소

교차 저장소 계약 (CALLS_API)  ✓
  프런트 API 호출 4 — 백엔드 라우트에 연결 4 (100%)
  동적 경로 1건 제외 — 런타임에 조립되어 정적 분석 불가
```

**"호출 연결률" 숫자 하나로 판단하지 마세요.** 분모에 외부 라이브러리 호출이
그대로 들어가므로 낮은 것이 정상입니다. 판단은 **미해소 호출 상위 목록**으로 합니다 —
거기 뜨는 이름이 외부 API면 정상이고, 우리 코드에 있어야 할 이름이면 추출기 결함입니다.

### ④ `nunchi serve` — MCP 등록

Claude Code에 등록:

```bash
claude mcp add nunchi -- /path/to/nunchi --config /path/to/nunchi.toml serve
```

또는 `.mcp.json`:

```json
{
  "mcpServers": {
    "nunchi": {
      "command": "/path/to/nunchi",
      "args": ["--config", "/path/to/nunchi.toml", "serve"]
    }
  }
}
```

---

## 질의

### `nunchi pack` — 가장 많이 쓰는 명령

태스크 한 문장으로 컨텍스트 팩을 만듭니다.

```bash
nunchi pack "댓글 삭제 로직 수정" --budget 4000
nunchi pack "주문 재시도" --json          # 에이전트/스크립트용
```

```
budget 4000 · used 2999 (99%)
seeds: should_delete_a_comment, delete, DELETE /articles/{}/comments/{}

tier     tok  symbol                       ref
L2       120  should_delete_a_comment      .../CommentServiceTest.java:137-142
L2       160  delete                       .../CommentController.java:53-60
L1       133  delete                       .../ArticleController.java:110-115
L1       132  DELETE /articles/{}          .../ArticleController.java:110-115
L0        63  getBySlug                    .../ArticleController.java:73-84

교차 저장소
  ✦ [api] DELETE /articles/{}/comments/{} — CommentController.java:53-60 (CALLS_API)
```

`tier`는 상세도입니다: **L2** 전체 본문 · **L1** 시그니처+문서+핵심 줄 · **L0** 좌표만.
예산이 모자라면 자동으로 강등됩니다.

### 그 외

```bash
nunchi find "OrderService" --limit 10      # 전문 검색
nunchi rules                               # 적용 중인 프레임워크 규칙
nunchi rules --toml                        # 그대로 복사해 확장
nunchi bench                               # 절감·recall 실측
nunchi tui                                 # 대화형 탐색·튜닝
```

### `nunchi bench` — 절감을 수치로

`bench/tasks.jsonl`에 실제 태스크를 한 줄에 하나씩 적습니다.

```jsonl
{"task":"댓글 삭제 로직 수정","expect":["CommentController.java","CommentService.java"]}
{"task":"주문 재시도","expect":["OrderService.java"]}
```

`expect`는 이 태스크를 풀려면 **반드시 봐야 하는 좌표**입니다(부분 경로 일치).

```
task                          grounded ungrounded    절감  recall
댓글 삭제 로직 수정                 2934       9593    69%    100%
게시글 조회                       3971      14889    73%    100%
사용자 인증 로그인                  3960       3462   -14%    100%
평균                            3647       9613    53%    100%
```

**두 가지를 함께 보세요.** 토큰만 줄고 `recall`이 떨어지면 무의미합니다.
그리고 절감이 음수인 태스크는 정상입니다 — 관련 코드가 적고 이름이 뚜렷하면
grep이 정확히 착지해서 그래프가 이기지 못합니다.

> `ungrounded`는 **대리 지표**입니다. 실제 에이전트 세션이 아니라
> "질의어가 걸리는 파일을 통째로 읽었을 때"를 계산합니다.
> 상대 비교에는 유효하지만 절대 절감률로 인용하지 마세요.

---

## TUI

```bash
nunchi tui
```

| 키 | 동작 |
|---|---|
| `tab` | 화면 전환 (탐색 / 영향범위 / 인덱스 / 팩 미리보기 / 지표) |
| `i` | 입력 모드 |
| `enter` | 실행 |
| `↑` `↓` | 항목 이동 (팩 화면에서는 가중치 선택) |
| `←` `→` | **가중치 조정 → 즉시 재랭킹** |
| `s` | 가중치를 `nunchi.toml`에 저장 |
| `q` | 종료 |

**④ 팩 미리보기**가 핵심 화면입니다. 슬라이더를 움직이면 랭킹이 즉시 다시 계산되고,
`s`로 저장하면 그 순간부터 에이전트도 같은 가중치를 씁니다.

각 화면은 서로 다른 고장을 잡습니다:

| 화면 | 잡아내는 문제 |
|---|---|
| ① 탐색 | **추출 오류** — 호출 엣지가 아예 없음 |
| ② 영향범위 | 영향 분석 누락 |
| ③ 인덱스 | **언어 커버리지** — "Kotlin 800개가 파싱 실패 중이었다" |
| ④ 팩 미리보기 | **랭킹 문제** |
| ⑤ 지표 | 교차 저장소 연결 회귀 |

---

## 확장 — 재빌드 없이

프레임워크 지원과 도메인 용어는 **설정 데이터**입니다. `nunchi.toml`에 추가하면
바이너리를 다시 만들지 않아도 적용됩니다.

### 사내 HTTP 래퍼

```toml
[[framework.http_client]]
lang = "typescript"
receiver_methods = ["fetchJson", "request"]
url_arg = 0
exclude_receivers = ["this", "app", "router"]   # 라우트 정의는 호출이 아니다
```

### 사내 어노테이션

```toml
[[framework.route]]
lang = "java"
annotation = "InternalEndpoint"
method = "POST"

[[framework.bean]]
lang = "java"
annotations = ["OurService", "OurComponent"]
```

### 사내 ORM · 매퍼

```toml
[[framework.persistence]]
lang = "java"
entity_annotations = ["OurEntity"]
table_annotations = ["OurTable"]
sql_annotations = ["OurQuery"]              # 어노테이션 안의 SQL에서 테이블을 뽑음
repository_supertypes = ["OurBaseRepository"]
```

### 파이썬 라우트 (사내 프레임워크)

```toml
[[framework.route]]
lang = "python"
annotation = "handler"                       # @our_app.handler("/x")
method = "POST"
receivers = ["our_app", "svc"]               # 이 수신자에서만 라우트로 본다
```

### 도메인 용어 사전

한국어로 물었을 때 영어 식별자에 닿게 합니다.

```toml
[semantic.terms]
# TOML은 비ASCII 키를 따옴표 없이 쓸 수 없습니다. 반드시 감싸세요.
"주문" = ["order", "orders"]
"결제" = ["payment", "billing", "charge"]
"회원" = ["user", "member", "account"]
```

현재 규칙을 보려면 `nunchi rules`, 그대로 복사하려면 `nunchi rules --toml`.

---

## 에이전트에게 쓰게 하기

각 저장소의 `CLAUDE.md`에 규칙을 넣으면 습관이 바뀝니다.

```markdown
## 코드 탐색

Grep/Glob으로 훑기 전에 `nunchi_pack`을 먼저 호출한다.
반환값은 답이 아니라 좌표(`path:line`)이므로, 지목된 범위만 Read한다.
`stale` 필드에 뜬 항목은 인덱스가 낡았다는 뜻이니 직접 Read한다.
```

배칭이 필요하면 MCP 대신 CLI를 씁니다 — 스키마 토큰이 들지 않고 한 번의 Bash 호출로
여러 질의를 묶을 수 있습니다:

```bash
nunchi pack "$TASK" --json && nunchi find "OrderService" --json
```

---

## 문제 해결

| 증상 | 원인과 대응 |
|---|---|
| `스키마 버전 불일치` | 추출기가 바뀌었습니다. `nunchi index --rebuild` |
| `인덱스가 없습니다` | `nunchi index`를 먼저 실행 |
| 팩에 엉뚱한 파일이 나옴 | `nunchi.toml`의 `exclude` 확인. 생성 코드가 들어왔을 가능성 |
| 호출 엣지가 거의 없음 | `nunchi doctor`의 미해소 상위 확인. 프레임워크 규칙 추가 필요 |
| 한국어 질의가 안 먹음 | `[semantic.terms]`에 용어 매핑 추가 |
| 팩 결과가 낡음 | `stale` 필드 확인. `nunchi index --watch`를 띄우거나 재인덱싱 |
| 지운 파일이 계속 나옴 | `nunchi index`가 자동 정리합니다. 그래도 남으면 `--rebuild` |

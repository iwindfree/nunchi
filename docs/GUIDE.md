# 사용 안내서

## 설치

```bash
git clone https://github.com/iwindfree/nunchi.git
cd nunchi
cargo build --release
```

산출물은 `target/release/nunchi`이며 Windows에서는 `nunchi.exe`입니다.
Rust 1.90 이상이 필요합니다. macOS와 Windows에서 각각 네이티브로 빌드하고
크로스 컴파일은 사용하지 않습니다.

경로에 등록해 두면 편리합니다.

```bash
cp target/release/nunchi ~/.local/bin/
```

---

## 최초 적용

네 단계로 진행합니다.

```bash
nunchi init ~/dev/order-api ~/dev/order-web --name web
nunchi index
nunchi doctor
nunchi serve
```

### 1단계: `nunchi init`

저장소를 훑어서 언어를 감지하고 설정 파일을 만듭니다. 여러 저장소를 하나의
**솔루션**으로 묶으면 그 사이에 교차 엣지가 생깁니다.

이 명령은 파일을 **두 개** 만듭니다.

| 파일 | 담는 내용 | 커밋 여부 |
|---|---|---|
| `nunchi.toml` | 저장소의 절대 경로이며 장비마다 다릅니다 | 커밋하지 않습니다 |
| `nunchi.shared.toml` | 랭킹 가중치, 프레임워크 규칙, 용어 사전, 제외 패턴 | 커밋합니다 |

공용 파일의 값이 장비별 파일의 값을 덮어씁니다. 업무 장비와 개인 장비가 같은
가중치와 같은 규칙을 사용하도록 하려면 `nunchi.shared.toml`을 커밋하시면
됩니다.

```toml
# nunchi.toml : 장비별 설정이며 커밋하지 않습니다
[solution]
name = "web"
repos = ["/Users/me/dev/order-api", "/Users/me/dev/order-web"]

[index]
languages = ["java", "typescript"]
max_file_bytes = 2097152
max_commits = 1000          # git 이력을 읽을 커밋 수이며 0이면 생략합니다
```

```toml
# nunchi.shared.toml : 경로가 들어가지 않으므로 장비 사이에서 공유됩니다
[rank]
alpha_bm25 = 0.7
beta_ppr = 0.5
gamma_recency = 0.3        # 반감기 30일 지수 감쇠를 적용합니다
delta_cochange = 0.4
epsilon_central = 0.2

[index]
exclude = ["**/node_modules/**", "**/build/**", "**/target/**"]
```

지원하는 언어는 `java`, `typescript`, `javascript`, `python`, `csharp`, `rust`입니다.

> **제외 패턴을 반드시 확인하십시오.** 생성된 코드나 벤더 디렉터리가 인덱스에
> 들어오면 랭킹이 오염되고 팩이 쓸모없는 결과를 반환합니다. 최초 적용에서
> 품질을 결정하는 요인의 절반이 여기에 있습니다.

### 2단계: `nunchi index`

```bash
nunchi index                # 증분 인덱싱이며 콘텐츠 주소 캐시를 사용합니다
nunchi index --rebuild      # 인덱스를 지우고 처음부터 다시 만듭니다
nunchi index --watch        # 데몬으로 실행하며 파일 변경을 감시합니다
```

`--watch`는 500밀리초 동안 변경을 모아서 처리합니다. 브랜치를 전환해도 이벤트가
폭주하지 않고, 원래 브랜치로 돌아올 때는 캐시가 적중하여 재파싱이 일어나지
않습니다.

인덱싱을 마치면 이번에 발견되지 않은 파일의 노드가 자동으로 제거되고, 이어서
아무도 참조하지 않게 된 의존성과 커밋, 저자 노드도 정리됩니다. 파일을 삭제하거나
이동했다는 이유로 `--rebuild`를 실행할 필요가 없습니다.

### 3단계: `nunchi doctor`

최초 적용에서 실제 관문이 되는 단계입니다.

```bash
nunchi doctor
nunchi doctor --json        # CI 게이트에서 사용합니다
```

첫 인덱싱은 대체로 "동작은 하지만 품질이 낮은" 상태로 끝납니다. 이 명령이 그
상태를 드러냅니다.

```
언어 커버리지
  java          80 files   80 파싱  100.0%  ✓
  javascript    50 files   50 파싱  100.0%  ✓
· sql           15 files       — 파서 없음

호출 연결률                     23.4%
  호출 1606 — 해소 199 · 모호 94 · 미해소 1161 · 후보과다 152
  미해소 호출 상위
    assertThat    79      AssertJ이므로 정상입니다
    save          45      JPA 리포지터리라서 본문이 없습니다
    builder       39      Lombok이 생성하는 코드이므로 정상입니다

프레임워크 의미론
  라우트 19 · Bean 32 · 주입 48해소/15미해소

교차 저장소 계약 (CALLS_API)  ✓
  프런트 API 호출 4 — 백엔드 라우트에 연결 4 (100%)
  동적 경로 1건 제외
```

**호출 연결률 숫자 하나만 보고 판단하지 마십시오.** 분모에 외부 라이브러리
호출이 그대로 포함되므로 이 값이 낮은 것은 정상입니다. 판단은 **미해소 호출
상위 목록**으로 하십시오. 거기에 나타나는 이름이 외부 API라면 정상이고,
우리 코드에 있어야 하는 이름이라면 추출기에 결함이 있다는 뜻입니다.

### 4단계: `nunchi serve`

Claude Code에 등록하는 방법은 두 가지입니다.

```bash
claude mcp add nunchi -- /path/to/nunchi --config /path/to/nunchi.toml serve
```

또는 `.mcp.json`에 직접 적으실 수 있습니다.

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

### `nunchi pack`

가장 자주 사용하는 명령입니다. 작업을 한 문장으로 적으면 컨텍스트 팩을
만들어 줍니다.

```bash
nunchi pack "댓글 삭제 로직 수정" --budget 4000
nunchi pack "주문 재시도" --json          # 에이전트나 스크립트에서 사용합니다
```

```
budget 4000 · used 2999 (99%)
seeds: should_delete_a_comment, delete, DELETE /articles/{}/comments/{}

tier     tok  symbol                       ref
L2       120  should_delete_a_comment      .../CommentServiceTest.java:137-142
L2       160  delete                       .../CommentController.java:53-60
L1       133  delete                       .../ArticleController.java:110-115
L0        63  getBySlug                    .../ArticleController.java:73-84

교차 저장소
  ✦ [api] DELETE /articles/{}/comments/{} — CommentController.java:53-60
```

`tier`는 상세도를 뜻합니다. **L2**는 전체 본문, **L1**은 시그니처와 문서 및
핵심 몇 줄, **L0**은 좌표만 담습니다. 예산이 부족해지면 자동으로 낮아집니다.

### 그 밖의 명령

```bash
nunchi find "OrderService" --limit 10
nunchi rules                     # 적용 중인 프레임워크 규칙을 봅니다
nunchi rules --toml              # 그대로 복사하여 확장할 수 있습니다
nunchi bench                     # 절감량과 재현율을 측정합니다
nunchi tui                       # 대화형으로 탐색하고 가중치를 조정합니다
```

### `nunchi bench`

`bench/tasks.jsonl`에 실제 작업을 한 줄에 하나씩 적습니다.

```jsonl
{"task":"댓글 삭제 로직 수정","expect":["CommentController.java","CommentService.java"]}
{"task":"주문 재시도","expect":["OrderService.java"]}
```

`expect`에는 그 작업을 해결하려면 반드시 확인해야 하는 좌표를 적습니다.
경로의 일부만 적어도 일치 판정이 됩니다.

```
task                          grounded ungrounded    절감  recall
댓글 삭제 로직 수정                 2934       9593    69%    100%
게시글 조회                       3971      14889    73%    100%
사용자 인증 로그인                  3960       3462   -14%    100%
평균                            3647       9613    53%    100%
```

**두 값을 함께 보십시오.** 토큰만 줄어들고 `recall`이 떨어지면 아무 의미가
없습니다. 그리고 절감량이 음수인 작업이 나오는 것은 정상입니다. 관련 코드가
적고 이름이 뚜렷하면 grep이 정확히 찾아내므로 그래프가 이기지 못합니다.

> `ungrounded` 값은 **대리 지표**입니다. 실제 에이전트 세션을 측정한 것이
> 아니고, 질의어에 걸리는 파일을 통째로 읽었을 때의 비용을 계산한 값입니다.
> 상대 비교에는 유효하지만 절대 절감률로 인용하지 마십시오.

---

## TUI

```bash
nunchi tui
```

| 키 | 동작 |
|---|---|
| `tab` | 화면을 전환합니다 (탐색, 영향 범위, 인덱스, 팩 미리보기, 지표) |
| `i` | 입력 모드로 들어갑니다 |
| `enter` | 실행합니다 |
| `↑` `↓` | 항목을 이동합니다. 팩 화면에서는 가중치를 선택합니다 |
| `←` `→` | 가중치를 조정하며 즉시 다시 계산됩니다 |
| `s` | 가중치를 `nunchi.shared.toml`에 저장합니다 |
| `q` | 종료합니다 |

**팩 미리보기 화면이 핵심입니다.** 가중치를 조정하면 랭킹이 즉시 다시
계산되고, 저장하면 그 시점부터 에이전트도 같은 가중치를 사용합니다.

화면마다 서로 다른 문제를 찾아냅니다.

| 화면 | 찾아내는 문제 |
|---|---|
| 탐색 | 추출 오류입니다. 호출 엣지가 아예 생기지 않은 경우를 발견합니다 |
| 영향 범위 | 영향 분석이 누락된 부분을 발견합니다 |
| 인덱스 | 언어 커버리지 문제입니다. 특정 언어가 전혀 파싱되지 않는 상황을 발견합니다 |
| 팩 미리보기 | 랭킹 문제입니다 |
| 지표 | 교차 저장소 연결이 이전보다 나빠졌는지 확인합니다 |

---

## 확장

프레임워크 지원과 도메인 용어는 **설정 데이터**입니다. `nunchi.shared.toml`에
추가하면 바이너리를 다시 빌드하지 않아도 적용됩니다.

### 사내 HTTP 래퍼

```toml
[[framework.http_client]]
lang = "typescript"
receiver_methods = ["fetchJson", "request"]
url_arg = 0
exclude_receivers = ["this", "app", "router"]   # 라우트 정의는 호출이 아닙니다
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

### 사내 ORM과 매퍼

```toml
[[framework.persistence]]
lang = "java"
entity_annotations = ["OurEntity"]
table_annotations = ["OurTable"]
sql_annotations = ["OurQuery"]              # 어노테이션 안의 SQL에서 테이블을 찾습니다
repository_supertypes = ["OurBaseRepository"]
```

### 파이썬 라우트

```toml
[[framework.route]]
lang = "python"
annotation = "handler"            # @our_app.handler("/x") 형태를 인식합니다
method = "POST"
receivers = ["our_app", "svc"]    # 이 수신자에서만 라우트로 판정합니다
```

### 도메인 용어 사전

한국어로 질의했을 때 영어 식별자에 닿도록 만듭니다.

```toml
[semantic.terms]
# TOML은 비ASCII 키를 따옴표 없이 쓸 수 없으므로 반드시 감싸야 합니다
"주문" = ["order", "orders"]
"결제" = ["payment", "billing", "charge"]
"회원" = ["user", "member", "account"]
```

현재 적용 중인 규칙은 `nunchi rules`로 확인하시고, 그대로 복사하려면
`nunchi rules --toml`을 사용하십시오.

---

## 에이전트가 사용하도록 만들기

각 저장소의 `CLAUDE.md`에 규칙을 넣으면 탐색 습관이 바뀝니다.

```markdown
## 코드 탐색

Grep이나 Glob으로 훑기 전에 `nunchi_pack`을 먼저 호출한다.
반환값은 답이 아니라 좌표(`path:line`)이므로 지목된 범위만 Read한다.
`stale` 필드에 나타난 항목은 인덱스가 낡았다는 뜻이니 직접 Read한다.
```

여러 질의를 묶어야 할 때는 MCP 대신 CLI를 사용하십시오. 스키마 비용이 들지
않고 Bash 한 번으로 여러 명령을 이어서 실행할 수 있습니다.

```bash
nunchi pack "$TASK" --json && nunchi find "OrderService" --json
```

---

## 문제 해결

| 증상 | 원인과 대응 |
|---|---|
| `스키마 버전 불일치` | 추출기가 바뀌었습니다. `nunchi index --rebuild`를 실행하십시오 |
| `인덱스가 없습니다` | `nunchi index`를 먼저 실행하십시오 |
| `nunchi.toml을 찾을 수 없습니다` | 설정 파일이 있는 디렉터리나 그 하위에서 실행하거나 `--config`로 지정하십시오 |
| 팩에 엉뚱한 파일이 나옵니다 | `exclude` 패턴을 확인하십시오. 생성된 코드가 들어왔을 가능성이 높습니다 |
| 호출 엣지가 거의 없습니다 | `nunchi doctor`의 미해소 상위 목록을 확인하고 프레임워크 규칙을 추가하십시오 |
| 한국어 질의가 결과를 내지 못합니다 | `[semantic.terms]`에 용어 대응을 추가하십시오 |
| 팩 결과가 낡았습니다 | `stale` 필드를 확인하십시오. `nunchi index --watch`를 실행하거나 다시 인덱싱하십시오 |
| 지운 파일이 계속 나옵니다 | `nunchi index`가 자동으로 정리합니다. 그래도 남으면 `--rebuild`를 실행하십시오 |
| 캐시 적중률이 계속 0%입니다 | Windows에서 `core.autocrlf` 설정 때문일 수 있습니다 |

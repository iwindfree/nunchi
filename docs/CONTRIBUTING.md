# 기여 안내서

이 문서는 코드를 고치거나 프로젝트를 이어받으려는 사람을 위한 것입니다.
사용 방법은 [사용 안내서](GUIDE.md)에, 작동 원리와 설계 근거는
[설계 문서](DESIGN.md)에 있습니다.

---

## 현재 상태

```
커밋 15개 · 코드 7,788줄 · 테스트 73개
언어 5개 (Java, TypeScript와 JavaScript, Python, C#, Rust)
RealWorld 저장소 실측: 평균 토큰 절감 53%, 재현율 100%
```

| 계층 | 상태 |
|---|---|
| 심볼 추출 (tree-sitter) | 언어 5개를 지원합니다 |
| 프레임워크 의미론 | Spring, FastAPI, Flask, ASP.NET, React를 지원합니다 |
| 영속 계층 | JPA, MyBatis(어노테이션과 XML), SQLAlchemy를 지원합니다 |
| 교차 저장소 연결 | 프런트엔드 호출에서 백엔드 라우트와 핸들러까지 이어집니다 |
| 컨텍스트 팩 | 랭킹, 토큰 예산, 상세도 강등을 모두 구현했습니다 |
| MCP 서버 | rmcp 3.2를 사용하며 도구 5개를 노출합니다 |
| TUI | 화면 5개를 제공합니다 |
| 벤치 | `nunchi bench`로 측정합니다 |
| SCIP 정밀 경로 | 착수하지 않았으며 남은 작업 중 가장 큽니다 |
| Windows 실측 | 검증하지 못했고 업무 장비에서만 가능합니다 |

---

## 빌드와 테스트

```bash
cargo build              # 개발용
cargo build --release    # 배포용 단일 바이너리
cargo test               # 전체 테스트
cargo test -p nunchi-core framework    # 모듈별 테스트
```

---

## 크레이트 구조

```
crates/
├── nunchi-core/                 라이브러리이며 모든 로직이 들어 있습니다
│   ├── model.rs                 노드 18종과 엣지 19종, NodeId 규약
│   ├── path.rs                  경로 정규화(Windows 대응)와 내용 해시
│   ├── lang.rs                  확장자에서 언어를 판별합니다
│   ├── config.rs                설정 파일 두 개를 읽고 병합합니다
│   ├── rules.rs                 규칙의 타입 정의와 병합·조회
│   ├── rules/                   내장 기본 규칙 (여기에 추가합니다)
│   │   ├── builtin.syntax.toml      언어별 구문 트리 노드 이름
│   │   ├── builtin.java.toml        Spring, JPA, MyBatis
│   │   ├── builtin.python.toml      FastAPI, Flask, SQLAlchemy
│   │   ├── builtin.typescript.toml  TypeScript · JavaScript
│   │   └── builtin.csharp.toml      ASP.NET, HttpClient
│   ├── semantic.rs              식별자 분해와 동의어 사전
│   ├── extract.rs               tree-sitter로 심볼을 추출합니다
│   ├── framework.rs             어노테이션과 데코레이터를 해석합니다
│   ├── mapper_xml.rs            MyBatis XML 매퍼를 읽습니다
│   ├── resolve.rs               이름에 기반하여 참조를 해소합니다
│   ├── history.rs               git 이력에서 동시 변경 관계를 만듭니다
│   ├── cache.rs                 내용 해시 기반 추출 캐시
│   ├── index.rs                 2패스 인덱싱을 조율합니다
│   ├── graph.rs                 메모리 그래프와 개인화 페이지랭크
│   ├── pack.rs                  랭킹과 토큰 예산 렌더링
│   ├── bench.rs                 벤치 하네스
│   ├── store/
│   │   ├── mod.rs               Store 트레이트이며 메서드가 6개입니다
│   │   └── sqlite.rs            SQLite(WAL)와 FTS5 구현
│   └── queries/*.scm            언어별 tree-sitter 쿼리
└── nunchi-cli/                  단일 바이너리 `nunchi`
    ├── main.rs                  서브커맨드
    ├── serve.rs                 MCP 서버
    ├── watch.rs                 파일 워처
    └── tui.rs                   ratatui로 만든 화면 5개
```

모든 로직은 `nunchi-core`에 있고 CLI는 진입점 역할만 합니다. MCP 서버와 CLI,
TUI가 같은 코어를 직접 호출하므로 로직이 중복되지 않습니다. 그 결과 TUI에
보이는 내용과 에이전트가 받는 내용이 동일합니다.

---

## 데이터 흐름

```
파일 워크 (ignore 크레이트를 쓰며 디렉터리 단위로 통째로 건너뜁니다)
   │
   ├─ 언어를 판별하고 내용 해시를 계산합니다
   │
   ├─ 캐시를 조회합니다 (해시와 언어로 조회합니다)
   │     ├─ 적중하면 재파싱하지 않습니다
   │     └─ 실패하면 tree-sitter로 파싱하고 캐시에 저장합니다
   │
   ├─ 1패스: File과 Symbol 노드, CONTAINS와 DEFINED_IN 엣지를 만듭니다
   │          Route와 ApiCall 노드, HANDLES 엣지를 만듭니다
   │          SymbolTable에 이름과 종류를 등록합니다
   │
   ├─ 2패스: 호출을 해소하여 CALLS를 만듭니다
   │          import를 해소하여 IMPORTS와 DEPENDS_ON을 만듭니다
   │          의존성 주입을 해소하여 INJECTS를 만듭니다
   │          경로를 대조하여 CALLS_API를 만듭니다 (교차 저장소)
   │          엔티티와 SQL에서 PERSISTS_TO를 만듭니다
   │
   ├─ git log에서 Commit과 Author, MODIFIED_BY, CO_CHANGED_WITH를 만듭니다
   │
   └─ 정리: 사라진 파일의 노드를 지우고, 끊어진 엣지를 지우고,
            고아 노드를 연쇄적으로 지웁니다
   │
   ▼
 SQLite (nodes, edges, nodes_fts, repos, meta)
   │
   ▼
 MemGraph::load → 페이지랭크 → pack::build_pack → 좌표 목록
```

2패스로 나눈 이유가 있습니다. 앞쪽 파일이 뒤쪽 파일의 심볼을 호출할 수 있으므로,
심볼을 모두 만든 다음에야 참조를 해소할 수 있습니다.

---

## 자주 하게 되는 작업

### 새 언어를 추가할 때

1. `Cargo.toml`에 `tree-sitter-<lang>`을 추가합니다.
2. `lang.rs`의 `detect()`에 확장자를 대응시키고 `is_code()`에 등록합니다.
3. `extract.rs`의 `SupportedLang`에 값을 추가하고 `language()`에 대응시킵니다.
4. `queries/<lang>.scm`을 작성합니다. 캡처 이름 규약은 다음과 같습니다.
   - `@def.<kind>`는 정의 노드 전체를 가리키며 위치 계산에 사용합니다.
   - `@name`은 그 정의의 이름입니다.
   - `@callee`는 호출 대상입니다.
   - `@import.path`는 import 경로입니다.
   - `@sub`와 `@super`는 상속 관계를 나타냅니다.
5. `all_queries_compile` 테스트에 그 언어를 추가합니다.

> 쿼리에 잘못된 노드 타입을 적으면 컴파일 시점이 아니라 실행 시점에 오류가
> 발생합니다. 반드시 `all_queries_compile` 테스트에 추가하십시오.

### 새 프레임워크를 지원할 때

두 가지 경우로 나뉩니다.

**자기 프로젝트에서만 쓸 규칙**이라면 코드를 고칠 필요가 없습니다.
`nunchi.shared.toml`의 `[[framework.*]]`에 추가하시면 됩니다. 방법은
[사용 안내서의 확장 절](GUIDE.md)에 정리해 두었습니다.

**모두가 쓸 기본 규칙**이라면 `crates/nunchi-core/rules/builtin.<언어>.toml`에
항목을 추가하고 `cargo test`를 실행하십시오. Rust 코드를 고칠 필요가
없습니다.

```toml
[[route]]
lang = "java"
annotation = "InternalEndpoint"
method = "POST"
receivers = []
```

각 필드가 무엇을 뜻하는지는 `rules.rs`의 구조체 주석에 적혀 있습니다.
현재 적용 중인 전체 규칙은 `nunchi rules --toml`로 출력할 수 있으므로,
자기 설정으로 검증한 규칙을 그대로 옮겨 오시면 됩니다.

> 규칙을 데이터 파일에 둔 이유가 있습니다. 규칙을 하나 더하는 일은 "이
> 어노테이션은 이 HTTP 메서드다"라는 사실을 적는 것뿐인데, Rust 코드로 두면
> `String` 변환과 `Vec` 생성 관용구를 알아야 했습니다. 프레임워크를 아는
> 사람이 Rust를 몰라서 기여하지 못할 이유가 없습니다.

필드 이름을 잘못 적어도 컴파일은 됩니다. TOML은 문자열이기 때문입니다.
`builtin_rules_parse` 테스트가 그것을 대신 잡아 줍니다. tree-sitter 쿼리와
`all_queries_compile`의 관계와 같습니다.

### 새 언어를 지원할 때 규칙 쪽에서 할 일

파일 두 개를 건드립니다.

1. `rules/builtin.syntax.toml`에 `[[lang_syntax]]`를 추가합니다. 그 언어의
   구문 트리에서 호출식과 문자열 리터럴이 무슨 이름인지 적는 자리입니다.
   이름을 알아내려면 짧은 코드를 파싱해 트리를 출력해 보면 됩니다.

   > 이 이름들은 tree-sitter 문법 크레이트가 정한 것이며 크레이트 버전에
   > 종속됩니다. 전부 1.0 이전이라 규칙 이름이 바뀔 수 있으므로, 문법
   > 크레이트를 올린 뒤에는 `cargo test`로 탐지가 여전히 동작하는지
   > 확인하십시오.
2. `rules/builtin.<언어>.toml`을 새로 만들고 `builtin()`의 `FILES` 목록에
   한 줄을 더합니다.

목록에 넣는 것을 잊으면 그 언어의 규칙이 통째로 사라지는데 컴파일은
그대로 됩니다. `builtin_covers_every_supported_language`가 그것을 잡습니다.

### 값과 절차의 경계

규칙 파일에는 **값**만 둡니다. 표에서 찾아보는 대상입니다. 트리를 훑고
URL을 뽑아 정규화하는 **절차**는 `framework.rs`에 둡니다.

이 경계를 지키면 새 프레임워크나 새 언어를 지원할 때 대부분 데이터만
고치게 됩니다. 기존 정형에 맞지 않는 새로운 형태가 필요할 때만 `rules.rs`에
규칙 종류를 추가하고 `framework.rs`에 처리 코드를 넣습니다.

### 랭킹을 조정할 때

가중치만 바꾸는 경우에는 `nunchi.shared.toml`이나 TUI에서 처리하시면 됩니다.
새로운 점수 항을 추가하려면 세 곳을 고쳐야 합니다.

1. `config.rs`의 `RankWeights`에 필드를 추가합니다.
2. `pack.rs`의 `build_pack` 점수 계산에 항을 추가합니다.
3. `tui.rs`의 `WEIGHT_LABELS`와 `adjust_weight`에 추가합니다.

세 곳을 모두 고치지 않으면 TUI 슬라이더가 값만 바꾸고 랭킹에는 영향을 주지
않는 상태가 됩니다. 실제로 그런 결함이 있었습니다.

### 저장 계층을 교체할 때

`store/mod.rs`의 `Store` 트레이트에 있는 메서드 6개만 구현하면 됩니다.
`sqlite.rs`가 참조 구현입니다. 이 트레이트를 좁게 유지하는 것이 교체 비용을
하루 이내로 묶어 두는 장치이므로, **메서드를 늘리기 전에 다시 생각하십시오.**

`SqliteStore`에는 트레이트 밖의 편의 메서드(`all_edges`, `files_by_lang` 등)도
있습니다. 이들은 SQLite 전용이므로 다른 백엔드로 옮길 때 대응이 필요합니다.

---

## 지켜야 하는 규약

### NodeId

```
repo:<repo>
file:<repo>/<path>
sym:<repo>/<path>#<name>
sym:<repo>#<name>                     partial 타입은 경로를 넣지 않습니다
route:<METHOD> <normalized-path>      솔루션 전역이며 저장소를 포함하지 않습니다
api:<repo>/<path>#<line>:<idx>
table:<name>                          솔루션 전역입니다
commit:<repo>/<sha>
author:<email>
dep:<name>
```

`route:`와 `table:`이 저장소 이름을 포함하지 않는 것이 중요합니다. 프런트엔드와
백엔드가 같은 노드를 가리켜야 `CALLS_API`가 성립합니다. C#의 `partial` 타입도
경로를 넣지 않는데, 한 타입이 여러 파일에 흩어져 있어서 경로를 포함하면 노드가
쪼개지기 때문입니다.

### 경로

- 저장하고 표시할 때는 항상 슬래시를 구분자로 쓰고 원래 대소문자를 보존합니다.
- 비교하고 조회할 때는 소문자 키를 사용합니다. NTFS가 대소문자를 구분하지 않기
  때문입니다.
- 파일을 읽을 때는 `path::to_extended_length`를 거칩니다. Windows의 260자 제한에
  대응하기 위해서입니다.
- 내용 해시는 워킹트리의 바이트를 기준으로 계산합니다. git의 blob SHA를 쓰면
  CRLF 환경에서 값이 달라집니다.

### 엣지

모든 엣지에 출처(`fast` 또는 `precise`)와 신뢰도를 붙입니다. 추정으로 만든
엣지에 1.0을 주지 마십시오. 이름이 일치한다는 사실은 타입을 해소한 것과
다릅니다.

### 로그

**`tracing`은 표준 오류로만 출력합니다.** stdio를 사용하는 MCP 서버에서 표준
출력은 JSON-RPC 메시지 전용이므로, 로그가 그곳에 섞이면 프로토콜이 깨집니다.
실제로 겪은 문제입니다.

---

## 테스트 전략

| 종류 | 위치 | 잡아내는 문제 |
|---|---|---|
| 쿼리 컴파일 | `extract.rs::all_queries_compile` | `.scm`의 잘못된 노드 타입 |
| 추출 동작 | `extract.rs`, `framework.rs` | 실제 프레임워크 코드 조각으로 검증합니다 |
| 오탐 | `route_definitions_are_not_client_calls` | 라우트 정의를 호출로 오인하는 문제 |
| 경로 정규화 | `normalizes_all_three_param_syntaxes` | 세 가지 표기가 같은 값이 되는지 |
| 참조 해소 | `resolve.rs` | 후보 수에 따른 신뢰도 분기 |
| 랭킹 | `graph.rs::pagerank_concentrates_near_seeds` | 시드 지배력과 거리에 따른 감쇠 |
| 저장 | `store/sqlite.rs` | 멱등성, FTS 특수문자 내성, 정리 동작 |

**추출기를 고칠 때는 실제 프레임워크 코드 조각으로 테스트를 작성하십시오.**
직접 만든 예제는 Lombok이나 어노테이션 조합 같은 실제 관용구를 놓칩니다.

### 실제 저장소로 검증하기

합성 테스트만으로는 부족합니다. RealWorld 저장소로 검증합니다.

```bash
git clone --depth 1 https://github.com/gabrielgua/realworld-springboot.git /tmp/rw/api
git clone --depth 1 https://github.com/romansndlr/react-vite-realworld-example-app.git /tmp/rw/web
cd /tmp/rw && nunchi init /tmp/rw/api /tmp/rw/web --name realworld
nunchi index && nunchi doctor
```

RealWorld는 하나의 API 명세를 여러 언어로 구현한 프로젝트 모음이므로,
Spring Boot 백엔드와 React 프런트엔드가 같은 계약을 공유합니다. `CALLS_API`를
검증하기에 이보다 적합한 공개 테스트베드를 찾기 어렵습니다.

현재 기대치는 다음과 같습니다.

```
java 80/80 파싱 · javascript 50/50 파싱
라우트 19 · Bean 32 · 주입 48해소
API 호출 4 — 라우트 연결 4 (100%) · 동적 1건 제외
```

---

## 이미 겪은 오류 세 건

개발 과정에서 **지표가 좋게 나왔는데 실제로는 틀렸던 경우가 세 번** 있었습니다.
세 번 모두 같은 교훈으로 이어집니다. 측정값을 믿기 전에 무엇을 세고 있는지
먼저 확인해야 합니다.

### 1. 교차 저장소 연결률 95%는 오탐 때문에 잘못 나온 값이었습니다

`CALLS_API` 연결률이 처음에 95%로 나왔습니다. 좋아 보였지만 틀린 값이었습니다.

탐지된 API 호출 21건 가운데 **16건이 miragejs 목 서버의 라우트 정의**였습니다.
`this.post('/users', handler)`는 클라이언트 호출이 아니라 서버가 라우트를
등록하는 코드입니다. 그런데 목 서버가 같은 API 명세를 그대로 반영하고 있었기
때문에 백엔드와 잘 연결되었고, 그 결과 지표가 실제보다 높게 나왔습니다.

대응으로 `exclude_receivers` 설정과 "인자에 함수가 있으면 핸들러 등록으로
판정한다"는 구조적 규칙을 넣었습니다. 후자는 프레임워크와 무관하게 작동합니다.
수정한 뒤 실제 클라이언트 호출은 4건이었고 연결도 4건이었습니다.

**교훈**: 지표가 예상보다 좋게 나오면 분자와 분모에 무엇이 들어갔는지 먼저
확인하십시오.

### 2. "심볼 해소율 95% 목표"는 도달할 수 없는 지표였습니다

초기 계획에 적었던 목표인데, 분모에 표준 라이브러리와 React, Spring 같은 외부
라이브러리 호출이 그대로 포함됩니다. 어떤 코드베이스에서도 95%가 나올 수
없습니다. 도달할 수 없는 목표를 제시하는 지표는 진단에 해롭습니다.

대응으로 이름을 "호출 연결률"로 바꾸고, 판단 근거가 되는 **미해소 호출 상위
목록**을 함께 출력하도록 했습니다. 그 목록에 나타나는 이름이 외부 API라면
정상이고, 우리 코드에 있어야 하는 이름이라면 추출기에 결함이 있다는 뜻입니다.
95% 목표는 그것이 실제로 유효한 SCIP 정밀 경로 지표로 옮겼습니다.

### 3. TESTS 엣지 628건은 많은 것이 좋은 것이 아니었습니다

호출에 기반해서만 만들었더니 628건이 생성되었는데, 대부분이 테스트 준비
코드에서 호출한 **Lombok 빌더의 필드 접근자**였습니다.
`setUp`에서 `body`, `title`, `description`으로 이어지는 엣지는 검증 대상이
아니라 DTO 필드를 가리킵니다.

대응으로 이름에 기반한 판정(`OrderServiceTest`에서 `OrderService`로 연결하며
신뢰도 0.9)을 주 경로로 삼고, 호출에 기반한 판정은 메서드와 함수, 생성자,
클래스로 제한했습니다(신뢰도 0.6). 그 결과 628건에서 156건으로 줄었고 남은
연결은 모두 의미가 있었습니다.

### 튜닝을 멈춘 지점

벤치의 "사용자 인증 로그인" 작업은 절감량이 -14%로 남아 있습니다. 그래프가
grep보다 나은 결과를 내지 못합니다. 원인을 확인했는데, 그 저장소의 인증 코드가 작은 파일
몇 개에 모여 있어서 grep이 정확히 찾아냅니다. 팩에 담긴 항목은 모두 실제로
인증과 관련된 코드였습니다.

**여기서 튜닝을 멈췄습니다.** 지표가 만족스러워질 때까지 계속 조정하는 것은 위의
세 사례와 같은 잘못입니다. 그래프가 grep에 미치지 못하는 경우를 드러내는 것도 벤치가
하는 일입니다.

이어받는 분께 부탁드립니다. 이 -14%를 고쳐야 할 결함으로 보지 마십시오.
고치려면 먼저 이유를 확인하시고, 그 이유가 정당하면 그대로 두시면 됩니다.

---

## 검증하지 못한 영역

### Windows

업무 장비가 Windows인데 **한 번도 실행해 보지 못했습니다.** 코드에는 대응이
들어 있지만 실측이 없습니다. 이어받으신 뒤 첫날에 하실 일입니다.

| 항목 | 문제가 발생할 가능성이 높은 이유 |
|---|---|
| 260자 경로 제한 | Spring의 깊은 패키지 구조와 Gradle의 `build/`가 겹치면 실제로 넘어갑니다. `path::to_extended_length`가 있지만 검증하지 못했습니다 |
| 워처 이벤트 폭주 | `ReadDirectoryChangesW`는 대량 변경을 처리하는 방식이 POSIX와 다릅니다. 브랜치 전환으로 재현해 보십시오 |
| SQLite WAL 동시 접근 | 파일 잠금 의미가 다릅니다. `index --watch`와 `serve`를 동시에 실행하여 확인하십시오 |
| CRLF 캐시 키 | `core.autocrlf`가 켜져 있으면 워킹트리와 blob이 달라집니다. 캐시 적중률이 계속 0%라면 이것을 의심하십시오 |

### 실제 솔루션에 적용하지 못했습니다

지금까지의 모든 수치는 **RealWorld 공개 저장소**를 대상으로 측정한 값입니다.
업무 코드에 적용한 적이 없습니다. 절감량과 연결률, 커버리지를 모두 다시
측정해야 합니다.

최초 적용 순서는 [사용 안내서](GUIDE.md)를 따르시되, `nunchi doctor`의 미해소
상위 목록을 반드시 눈으로 확인하십시오. 그 목록에 사내 관용구가 나타나면
`nunchi.shared.toml`에 규칙을 추가하시면 됩니다. 바이너리를 다시 빌드할 필요가
없습니다.

---

## 남은 작업

우선순위 순으로 정리했습니다.

1. **Windows 실측**입니다. 위에 적은 항목 네 개를 확인해야 하며, 다른 모든
   작업의 전제가 됩니다.
2. **실제 솔루션에 적용하고 벤치를 실행하는 일**입니다. 업무 작업으로 다시
   측정해야 합니다. `bench/tasks.jsonl`에 실제 업무 15개에서 20개를 적어 두면
   가장 유용한 자료가 됩니다.
3. **SCIP 정밀 경로 연동**입니다(`scip-java`, `scip-typescript`). 남은 작업 중
   가장 큽니다. 현재는 이름에 기반한 추정만 있습니다. 빌드가 필요하므로 커밋과
   CI, 유휴 시간에만 실행하는 2단 구조로 설계해 두었습니다. `Provenance`의
   `Fast`와 `Precise` 구분이 이미 준비되어 있습니다.
4. **JPA 파생 쿼리 해석**입니다. `findByStatusAndCreatedAtAfter` 같은 메서드
   이름을 해석하여 엔티티 필드로 연결해야 합니다.
5. **호출에 기반한 라우팅 지원**입니다. Django의 `path(...)`와 Express의
   `app.get(path, handler)`를 다루려면 규칙 모델에 축을 하나 더 늘려야 합니다.
6. **메서드를 이어 부르는 HTTP 클라이언트 지원**입니다. Spring `WebClient`의
   `get().uri("/api/x")`와 OkHttp의 요청 빌더는 URL이 별도 호출의 인자로
   들어가므로 지금 구조로는 읽지 못합니다. 호출 사슬을 거슬러 올라가며
   메서드와 경로를 모으는 처리가 필요합니다.
7. **WinForms 심화**입니다. Designer 파일의 이벤트 배선(`btnSave.Click += ...`)을
   `HANDLES` 엣지로 만들어야 합니다. `partial` 병합은 이미 완료했습니다.
8. **파일 단위 증분 갱신**입니다. 워처가 아직 전체를 다시 인덱싱합니다. 캐시
   덕분에 비용은 낮습니다.

---

## 개발 환경

| | 개인 장비 (macOS) | 업무 장비 (Windows) |
|---|---|---|
| 역할 | 개발과 RealWorld 테스트, 자체 인덱싱 | 실측과 실사용 |
| 빌드 | `cargo build --release` | 동일하며 크로스 컴파일은 하지 않습니다 |
| 인덱스 | 각자 자기 코드만 인덱싱하며 동기화하지 않습니다 | |

저장소는 https://github.com/iwindfree/nunchi 입니다.

**업무 코드는 저장소 밖으로 나가지 않습니다.** 오가는 것은 도구의 소스 코드와
`nunchi doctor --json`이 출력하는 통계뿐입니다.

설정 파일 두 개의 구분을 지켜 주십시오.

- `nunchi.toml`은 저장소의 절대 경로를 담고 장비마다 다르므로 커밋하지 않습니다.
- `nunchi.shared.toml`은 가중치와 규칙, 용어 사전을 담으므로 커밋하여 양쪽 장비가
  같은 값을 쓰게 합니다.

---

## 이어받는 첫날

```bash
git clone https://github.com/iwindfree/nunchi.git && cd nunchi
cargo test                                    # 테스트 73개가 통과하는지 확인합니다
cargo build --release

# 자체 인덱싱으로 동작을 익힙니다
./target/release/nunchi init . --name nunchi
./target/release/nunchi index
./target/release/nunchi doctor
./target/release/nunchi pack "랭킹 가중치 조정" --budget 3000
./target/release/nunchi tui                   # 팩 미리보기에서 가중치를 조정해 보십시오
```

그다음 [설계 문서](DESIGN.md)를 읽으시고, 위에 적은 오류 세 건을 기억하신 채로
Windows 실측으로 넘어가시면 됩니다.

# 개발 문서

원리는 [PRINCIPLES.md](PRINCIPLES.md), 사용법은 [USAGE.md](USAGE.md),
설계 결정의 배경은 저장소 루트의 [PLAN.md](../PLAN.md)에 있습니다.
이 문서는 **코드를 고치려는 사람**을 위한 것입니다.

## 빌드와 테스트

```bash
cargo build              # 개발
cargo build --release    # 배포용 단일 바이너리
cargo test               # 전체
cargo test -p nunchi-core framework    # 모듈별
```

Rust 1.90 이상. macOS와 Windows에서 각각 네이티브 빌드합니다(크로스 컴파일 안 함).

---

## 크레이트 구조

```
crates/
├── nunchi-core/                 라이브러리 — 모든 로직
│   ├── model.rs                 노드 18종 / 엣지 19종, NodeId 규약
│   ├── path.rs                  경로 정규화 (Windows 대응), 내용 해시
│   ├── lang.rs                  확장자 → 언어
│   ├── config.rs                nunchi.toml
│   ├── rules.rs                 프레임워크 규칙 (데이터)
│   ├── semantic.rs              식별자 분해, 동의어
│   ├── extract.rs               tree-sitter 심볼 추출
│   ├── framework.rs             Spring 어노테이션 / React API 호출
│   ├── resolve.rs               이름 기반 참조 해소
│   ├── history.rs               git 이력 → 동시변경
│   ├── cache.rs                 콘텐츠 주소 추출 캐시
│   ├── index.rs                 2패스 인덱싱 오케스트레이션
│   ├── graph.rs                 메모리 그래프 + PPR
│   ├── pack.rs                  랭킹 + 토큰 예산 렌더링
│   ├── store/
│   │   ├── mod.rs               Store 트레이트 (6개 메서드)
│   │   └── sqlite.rs            SQLite(WAL) + FTS5
│   └── queries/*.scm            tree-sitter 쿼리 (언어별)
└── nunchi-cli/                  단일 바이너리 `nunchi`
    ├── main.rs                  서브커맨드
    ├── serve.rs                 MCP 서버 (rmcp)
    ├── watch.rs                 파일 워처
    └── tui.rs                   ratatui 5화면
```

**모든 로직은 `nunchi-core`에 있습니다.** CLI는 얇은 진입점입니다.
MCP 서버·CLI·TUI가 같은 코어를 직접 호출하므로 로직 중복이 없고,
TUI에 보이는 것이 에이전트가 받는 것과 동일합니다.

---

## 데이터 흐름

```
파일 워크 (ignore, 디렉터리 가지치기)
   │
   ├─ 언어 판별 → 내용 해시
   │
   ├─ [캐시 조회] hash+lang → FileFacts?
   │     ├─ 적중 → 재파싱 없음
   │     └─ 미스 → tree-sitter 파싱 → 캐시 저장
   │
   ├─ 1패스: File·Symbol 노드, CONTAINS/DEFINED_IN 엣지
   │          Route·ApiCall 노드, HANDLES 엣지
   │          SymbolTable에 이름 등록
   │
   └─ 2패스: 호출 해소 → CALLS
             import 해소 → IMPORTS / DEPENDS_ON
             DI 해소     → INJECTS
             경로 매칭   → CALLS_API      ← 교차 저장소
   │
   └─ git log → Commit/Author, MODIFIED_BY, CO_CHANGED_WITH
   │
   └─ 정리: 사라진 파일의 노드 → 끊긴 엣지 → 고아 노드(연쇄)
   │
   ▼
 SQLite (nodes, edges, nodes_fts, repos, meta)
   │
   ▼
 MemGraph::load → PPR → pack::build_pack → 좌표 목록
```

**2패스인 이유**: 앞 파일이 뒤 파일의 심볼을 호출할 수 있으므로,
심볼을 전부 만든 뒤에야 해소할 수 있습니다.

---

## 자주 하게 될 작업

### 새 언어 추가

1. `Cargo.toml`에 `tree-sitter-<lang>` 추가
2. `lang.rs`의 `detect()`에 확장자 매핑, `is_code()`에 등록
3. `extract.rs`의 `SupportedLang`에 variant 추가 + `language()` 매핑
4. `queries/<lang>.scm` 작성 — 캡처 규약:
   - `@def.<kind>` — 정의 노드 전체 (span 계산에 사용)
   - `@name` — 그 정의의 이름
   - `@callee` — 호출 대상
   - `@import.path` — import 경로
5. `all_queries_compile` 테스트가 쿼리 오류를 잡아줍니다

> 쿼리의 잘못된 노드 타입은 **런타임에야** 터집니다.
> 반드시 `all_queries_compile`에 언어를 추가하세요.

### 새 프레임워크 지원

대부분 **코드를 고칠 필요가 없습니다.** `nunchi.toml`의 `[[framework.*]]`에
규칙을 추가하면 됩니다 ([USAGE.md](USAGE.md#확장--재빌드-없이)).

기존 정형(어노테이션→라우트/Bean/주입, 호출→ApiCall)에 안 맞는 새로운 모양이
필요할 때만 `rules.rs`에 규칙 종류를 추가하고 `framework.rs`에 처리를 넣습니다.

### 랭킹 조정

가중치만 바꾸는 것이면 `nunchi.toml`이나 TUI에서 하면 됩니다.
새로운 점수 항을 추가하려면:

1. `config.rs`의 `RankWeights`에 필드 추가
2. `pack.rs`의 `build_pack` 점수 계산에 항 추가
3. `tui.rs`의 `WEIGHT_LABELS`와 `adjust_weight`에 추가

### 저장 계층 교체

`store/mod.rs`의 `Store` 트레이트 6개 메서드만 구현하면 됩니다.
`sqlite.rs`가 참조 구현입니다. 이 트레이트를 좁게 유지하는 것이
교체 비용을 하루 이내로 묶는 장치이므로, **메서드를 늘리기 전에 다시 생각하세요.**

`SqliteStore`에는 트레이트 밖의 편의 메서드(`all_edges`, `files_by_lang` 등)도
있습니다. 이들은 SQLite 전용이므로 다른 백엔드로 갈 때 대응이 필요합니다.

---

## 지켜야 할 규약

### NodeId

```
repo:<repo>
file:<repo>/<path>
sym:<repo>/<path>#<name>
route:<METHOD> <normalized-path>      ← 솔루션 전역. 저장소가 달라도 같은 노드
api:<repo>/<path>#<line>:<idx>
commit:<repo>/<sha>
author:<email>
dep:<name>
```

`route:`가 저장소를 포함하지 않는 것이 핵심입니다. 프런트와 백엔드가
**같은 노드를 가리켜야** `CALLS_API`가 성립합니다.

### 경로

- 저장·표시는 항상 **슬래시 구분자, 원본 대소문자 보존** (`path::normalize`)
- 비교·조회는 **소문자 키** (`path::compare_key`) — NTFS 대소문자 비구분 대응
- 파일 읽기는 `path::to_extended_length` 경유 — Windows MAX_PATH 260 대응
- 내용 해시는 **워킹트리 바이트** 기준. git blob SHA를 쓰면 CRLF 환경에서 갈립니다

### 엣지

모든 엣지에 `provenance`(`fast`/`precise`)와 `confidence`를 답니다.
휴리스틱으로 만든 엣지에 1.0을 주지 마세요 — 이름 일치는 타입 해소가 아닙니다.

### 로그

**`tracing`은 stderr로만 씁니다.** stdio MCP 서버에서 stdout은 JSON-RPC 전용
채널이라 로그가 프로토콜을 깨뜨립니다. (실제로 겪은 버그입니다.)

---

## 테스트 전략

| 종류 | 위치 | 무엇을 잡나 |
|---|---|---|
| 쿼리 컴파일 | `extract.rs::all_queries_compile` | .scm의 잘못된 노드 타입 |
| 추출 동작 | `extract.rs`, `framework.rs` | 실제 Spring/React 코드 조각으로 검증 |
| 오탐 | `route_definitions_are_not_client_calls` | `this.post()` 같은 라우트 정의 |
| 경로 정규화 | `framework.rs::normalizes_all_three_param_syntaxes` | `{id}` ≡ `:id` ≡ `${id}` |
| 해소 | `resolve.rs` | 후보 수에 따른 confidence 분기 |
| 랭킹 | `graph.rs::pagerank_concentrates_near_seeds` | 시드 지배력, 거리 감쇠 |
| 저장 | `store/sqlite.rs` | 멱등성, FTS 메타문자 내성 |

**추출기를 고칠 때는 실제 프레임워크 코드 조각으로 테스트를 쓰세요.**
합성 예제는 실제 관용구(Lombok, 어노테이션 조합)를 놓칩니다.

### 실제 저장소로 검증

합성 테스트만으로는 부족합니다. RealWorld로 검증합니다:

```bash
git clone --depth 1 https://github.com/gabrielgua/realworld-springboot.git /tmp/rw/api
git clone --depth 1 https://github.com/romansndlr/react-vite-realworld-example-app.git /tmp/rw/web
cd /tmp/rw && nunchi init /tmp/rw/api /tmp/rw/web --name realworld && nunchi index && nunchi doctor
```

RealWorld는 **하나의 API 명세를 여러 언어로 구현**한 프로젝트라
Spring Boot 백엔드와 React 프런트가 같은 계약을 공유합니다.
`CALLS_API` 검증에 이보다 맞는 공개 테스트베드가 없습니다.

기대치(현재):

```
java 80/80 파싱 · javascript 50/50 파싱
라우트 19 · Bean 32 · 주입 48해소
프런트 API 호출 4 — 연결 4 (100%) · 동적 1건 제외
```

---

## Windows에서 확인할 것

회사 장비가 Windows이므로 다음은 **반드시 실측**해야 합니다
(PLAN.md 3.10절):

| 항목 | 왜 |
|---|---|
| 긴 경로 (MAX_PATH 260) | Spring 깊은 패키지 + Gradle `build/` 조합에서 실제로 걸립니다 |
| 워처 이벤트 폭풍 | `ReadDirectoryChangesW`는 대량 변경 동작이 다릅니다 |
| SQLite WAL 동시 접근 | 파일 잠금 의미가 POSIX와 다릅니다 |
| CRLF | `core.autocrlf` 환경에서 캐시 키가 예상대로 동작하는지 |

---

## 알려진 한계와 다음 작업

우선순위 순:

1. **SCIP 정밀 경로** — `scip-java`, `scip-typescript` 연동. 빌드가 필요하므로
   커밋/CI/유휴 시에만 돌리는 2단 구조다. 현재는 tree-sitter 이름 기반 휴리스틱뿐
2. **JPA 파생 쿼리** — `findByStatusAndCreatedAtAfter` 같은 메서드 이름을
   해석해 Entity 필드로 잇기
3. **호출 기반 라우팅** — Django `path(...)`, Express `app.get(path, handler)`.
   규칙 모델에 축을 하나 더 늘려야 한다
4. **파일 단위 증분 갱신** — 워처가 아직 전체 재인덱싱(캐시로 비용은 낮음)
5. **WinForms 심화** — Designer 이벤트 배선(`btnSave.Click += ...`) → HANDLES,
   `.resx` 리소스. partial 병합은 완료
6. **Windows 실측** — 긴 경로·워처 폭풍·WAL. 회사 장비에서만 가능
7. **Doc/Contract 노드** — 문서 연결, DTO 계약 기반 교차 저장소 엣지

### 현재 지원 범위

| 언어 | 심볼 | 라우트 | DI | 영속 |
|---|---|---|---|---|
| Java | ✅ | Spring | ✅ | JPA · MyBatis(어노테이션·XML) |
| TypeScript/JS | ✅ | react-router(부분) | — | — |
| Python | ✅ | FastAPI · Flask | — | SQLAlchemy |
| C# | ✅ (partial 병합) | ASP.NET | — | — |
| Rust | ✅ | — | — | — |

HTTP 클라이언트(`CALLS_API`의 프런트 끝): fetch·axios(TS/JS), requests·httpx(Python),
HttpClient(C#).

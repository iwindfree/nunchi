# Context Graph 도입 계획

> 목표: 여러 솔루션 저장소에 대해 "미리 계산된 컨텍스트 그래프"를 구축하고,
> 코딩 에이전트가 파일을 대량으로 읽는 대신 그래프에 질의해 **압축된 사실 + 정확한 좌표**를
> 받도록 전환한다.


## 용어

**`nunchi`** — 이 도구의 이름. 한국어 **눈치**(말해지지 않은 맥락을 읽는 능력)에서 왔다.
에이전트가 grep으로는 볼 수 없는 배선(`@Autowired`, `@GetMapping`, 프런트–백엔드 계약)을
읽게 해주는 것이 이 도구의 일이므로 이름이 기능과 그대로 대응한다. 산출물은 서브커맨드를 가진 **단일 정적 실행 파일 하나**다.

| 명령 | 역할 | 주 사용자 |
|---|---|---|
| `nunchi init` | `nunchi.toml` 생성, 저장소·언어 감지 (3.8절) | 사람 (온보딩 1회) |
| `nunchi index [--watch]` | 인덱싱 / 워처 데몬. **쓰기 소유** (3.5·3.6절) | 데몬 |
| `nunchi doctor [--json]` | 품질 검증 — 커버리지·심볼 해소율 (3.8절) | 사람 · CI |
| `nunchi serve` | **MCP 서버** (stdio) | 에이전트 |
| `nunchi find` / `nunchi pack` / `nunchi impact` … | CLI 질의, JSON 출력 (3절) | 에이전트(배칭) · 사람 |
| `nunchi tui` | 그래프 탐색·팩 미리보기·가중치 튜닝 (1.6절) | 사람 |
| `nunchi bench` | grounded vs ungrounded 실측 (Phase 0) | 사람 |

**`nunchi_find` / `nunchi_pack` / `nunchi_neighbors` / `nunchi_impact` / `nunchi_recent`** (밑줄 표기)
— `nunchi serve`가 에이전트에게 노출하는 **MCP 툴 5개**의 이름(3절).
동일 기능을 CLI로도 제공하며 이름을 맞춰 두었다.

**`nunchi.toml`** — 솔루션별 설정. 저장소 경로, 제외 패턴, 랭킹 가중치 α~ε (3.8·3.10절).

---

## 0. 왜 그래프인가 (Uber 사례에서 실제로 가져올 것)

| 항목 | Uber | 우리 목표치(초기) |
|---|---|---|
| 규모 | 24M 노드 / 80M 엣지, 86 노드타입 / 117 엣지타입 | ~1M 노드, 18 노드타입 / 19 엣지타입 |
| 소스 | 30+ 내부 시스템 | 코드 + git + 문서 + 설정 |
| 효과 | 플릿 전체 토큰 40%+ 절감, 1,000+ 툴 접근 | 태스크당 입력 토큰 40~60% 절감 |
| 대조 실험 | grounded 38초 / ungrounded 20분+·오답 | 동일 방식으로 측정 |

**중요한 해석**: Uber의 40% 절감은 그래프 단독 효과가 아니라
(1) 컨텍스트 그래프 (2) MCP 게이트웨이(툴 스키마를 컨텍스트에서 제거)
(3) 프롬프트 캐싱 TTL 튜닝 (4) code-mode 배칭 (5) 모델 라우팅
다섯 개 레버의 합이다. 우리도 다섯 개를 모두 다루되, **(2)(3)은 즉시 적용 가능한
공짜 이득**이고 (1)이 개발 대상이다.

### 토큰이 새는 지점 (세션 실측)

```
System tools           24.3k
MCP tools (active)      9.8k
MCP tools (deferred)   67.7k   ← 지연 로딩으로 이미 절약 중
Skills                  6.0k
```

대화 시작 시점에 **이미 ~40k**가 도구 스키마로 소모된다. 여기에 에이전트가
코드베이스를 탐색하며 읽는 파일이 태스크당 20~100k씩 얹힌다.
그래프가 겨냥하는 건 후자다.

---

## 1. 시스템 개요

```
 ┌──────────────┐   ┌──────────────┐   ┌──────────────┐
 │ spring-boot  │   │ react-web    │   │ winforms-app │   (임의 경로, config로 지정)
 └──────┬───────┘   └──────┬───────┘   └──────┬───────┘
        └──────────────────┼──────────────────┘
                           ▼
                 ┌───────────────────┐
                 │  Extractors       │  tree-sitter / SCIP / git / md / config
                 │  (증분, 파일해시)  │
                 └─────────┬─────────┘
                           ▼
                 ┌───────────────────┐
                 │  Graph Store      │  SQLite (+FTS5, +sqlite-vec)
                 │  nodes/edges/text │  → 어댑터 뒤. 필요 시 LadybugDB/Neo4j
                 └─────────┬─────────┘
                           ▼
                 ┌───────────────────┐
                 │  Retrieval/Rank   │  BM25 × 그래프 근접도 × 변경이력
                 │  Context Packer   │  토큰 예산 내 렌더링(L0/L1/L2)
                 └─────────┬─────────┘
              ┌───────────┼───────────┐
              ▼           ▼           ▼
        ┌──────────┐ ┌──────────┐ ┌──────────┐
        │ MCP 서버 │ │ CLI (nunchi) │ │   TUI    │  ← 사람용. 그래프 탐색·
        │ (에이전트)│ │ (배칭용) │ │(ratatui) │     팩 미리보기·가중치 튜닝
        └──────────┘ └──────────┘ └──────────┘
        단일 정적 바이너리 하나에 서브커맨드 (`nunchi serve` / `nunchi` / `nunchi tui`)
```

**핵심 설계 원칙**

1. **그래프는 답을 주지 않는다. 좌표를 준다.** 반환값은 요약 + `path:line` 이며,
   에이전트가 필요할 때만 해당 범위를 Read한다. 이게 토큰 절감의 본체다.
2. **MCP 툴 개수를 최소화한다.** 툴 5개 이하. 스키마가 곧 상시 비용이다.
3. **CLI를 1급으로 만든다.** 에이전트가 Bash 한 번으로 여러 질의를 배칭하면
   턴 수와 스키마 토큰을 동시에 줄인다 (Uber code-mode, 50~90% 절감 구간).
4. **증분 인덱싱.** 파일 해시 기반. 전체 재빌드는 절대 기본 경로가 되면 안 된다.

---

## 1.5 선행 사례 — 참고할 것과 넘어설 기준선

**결정: 직접 만든다.** 기성 도구는 채택 대상이 아니라 *벤치마크 기준선*으로 쓴다.

> **용어 정리**: CodeGraph는 DB가 아니라 **MCP 서버 + 인덱서 완제품**이며 저장은
> 내부적으로 임베디드 SQLite를 쓴다. 아래 계층 중 ②③④를 묶은 응용이다.
> 2절의 저장소 논의는 ① 계층 문제로, 직접 만들더라도 그대로 남는다.
>
> | 계층 | 선택지 |
> |---|---|
> | ④ 서빙 | MCP / CLI / TUI |
> | ③ 검색·랭킹 | BM25 / 벡터 / 그래프 근접도 |
> | ② 추출 | tree-sitter / SCIP / LSP |
> | ① 저장 | SQLite · LadybugDB · Neo4j · FalkorDB |

### 기준선 (우리 벤치에서 이 수치를 넘겨야 자체 개발이 정당화된다)

| 도구 | 구성 | 공개 수치 | 약점 (= 우리 기회) |
|---|---|---|---|
| CodeGraph | 임베디드 SQLite + tree-sitter, 21개 언어, 파일워처 증분 | 툴콜 58%(벤더)/70%(독립) 감소 | 시맨틱 검색 없음, **교차 저장소 없음**, 단독 메인테이너(커밋 ~91%) |
| GitNexus | LadybugDB, import·call·definition | 툴콜 88%, **토큰 74% 절감**(프로덕션 감사) | PolyForm Noncommercial 라이선스, 증분 갱신 미구현 |
| Serena | LSP 기반 40+ 언어, 기여자 170+ | 독립 벤치 없음 | 심볼 단위만, 전체 의존성 그래프 없음 |
| CodeGraphContext | Neo4j 서버 백엔드 | — | 서버 의존 |

> 위 수치는 벤더 발표 또는 3자 비교글 기준이며 **우리 저장소에서 재현 검증 전이다.**
> 또한 "codegraph"라는 이름의 별개 프로젝트가 최소 4개 존재한다
> (`sdsrss/code-graph-mcp`, `CodeGraphContext`, `DeusData/codebase-memory-mcp`,
> `Phoenixrr2113/codebase-graph`) — 참조 시 리포지터리를 명시할 것.

### 자체 개발로만 가능한 차별점 (설계 우선순위)

1. **교차 저장소 연결** — React ↔ Spring Boot를 API 계약으로 잇는 것(3.9절).
   임베디드 계열은 대부분 단일 저장소 인덱서라 기성품으로는 원리적으로 불가능하다.
2. **`nunchi_pack` 토큰 예산 컨텍스트 패킹** — 3자 비교에서도 별도 카테고리
   (Repomix, code2prompt)로 분류되며, 구조 그래프와 결합된 형태는 사실상 없다.
   **토큰 절감의 본체이자 가장 큰 차별점.**
3. **git 동시변경 결합도 + 문서·ADR 연결** — 영향 범위 분석의 나머지 절반.
4. **랭킹 튜닝 루프** — 우리 태스크 벤치로 가중치를 조정하는 구조.
   기성품은 남의 코드베이스에 맞춰진 고정 휴리스틱이다.

### 참고할 구현 패턴 (바퀴 재발명 금지)

- tree-sitter 파서 번들링·언어 등록 방식 → Serena, Codebase-Memory
- 콘텐츠 해시 기반 증분 재인덱싱 + 파일워처 → Codebase-Memory
- MCP 툴 표면을 얇게 유지하는 법(`context` / `explore` 2개로 압축한 사례)
- bitemporal(`valid_at`/`invalid_at`) 노드 — 시점 질의가 필요해지면 채택 검토(3.7절)

---

## 1.6 구현 스택 — Rust + TUI

### Rust를 택하는 이유

| 근거 | 내용 |
|---|---|
| **tree-sitter가 Rust 네이티브** | 참조 구현이 Rust. WASM·node-gyp 바인딩 층이 통째로 사라진다 |
| **단일 정적 바이너리, 의존성 0** | 여러 솔루션 저장소에 배포·고정하기 압도적으로 쉽다. 선례: `DeusData/codebase-memory-mcp`가 정확히 이 구성(단일 정적 바이너리, 158개 언어, 밀리초 단위 인덱싱) |
| **MCP SDK 위험 해소** | 공식 Rust SDK `rmcp`가 **2026-08-21 Tier 1 승격**, 3.0.1 안정 릴리스, 서버 conformance 67/67·클라이언트 50/50 통과. stdio·HTTP+SSE 모두 지원 |
| **메모리 상주 그래프에 최적** | 인접리스트 + PPR power iteration은 GC 없는 언어에서 확연히 유리. 2절에서 순회를 메모리로 옮긴 설계와 맞물린다 |
| **저장 계층 양쪽 다 지원** | `rusqlite` 성숙, LadybugDB도 Rust SDK 제공 → 2절 스파이크를 양쪽 다 Rust로 측정 가능 |

**비용과 대응**: Phase 2 랭킹 튜닝은 반복 실험이 많아 컴파일 사이클이 부담이다.
→ 가중치 α~ε를 코드가 아닌 **`nunchi.toml` 설정으로 분리**해 재컴파일 없이 조정하고,
아래 TUI에서 실시간으로 만진다.

### TUI — 임베디드 선택으로 잃은 것을 되찾는 자리

2절에서 임베디드 DB를 고르며 "Neo4j Browser급 시각화 상실"을 비용으로 인정했다.
TUI가 그 공백을 메운다. `ratatui`로 구현하고 **동일 스토어 어댑터 위의 얇은 뷰어**로
유지한다(별도 코드베이스 금지).

**화면 구성**

1. **탐색** — 심볼 검색 → 이웃 드릴다운(파일 브라우저식 좌우 패널)
2. **영향 범위** — `nunchi_impact` 결과를 트리뷰로. 호출자·테스트·동시변경 파일
3. **인덱스 상태** — 진행률, 언어별 커버리지, stale 노드 비율, 재인덱싱 트리거
4. **팩 미리보기** ★ — `nunchi_pack` 결과를 **토큰 수와 함께** 표시하고
   가중치 α~ε 슬라이더를 실시간 조정. **가장 값어치 있는 화면이다.**
5. **벤치 대시보드** — grounded vs ungrounded 토큰·턴·정답률 비교

**하지 않을 것**: 실제 그래프 다이어그램 렌더링. 터미널에서 노드-엣지 레이아웃은
가독성이 나오지 않는다. 노드 중심의 리스트·트리 탐색으로 설계한다.

### ⚠️ 범위 경고 — TUI는 임계 경로가 아니다

이 시스템의 **주 소비자는 사람이 아니라 에이전트**이고, 제품 본체는 MCP 서버 + CLI다.
TUI를 Phase 1에 넣으면 실제 토큰 절감을 만드는 `nunchi_pack`(Phase 2)이 밀린다.
→ **TUI는 Phase 3.5에 배치**한다. 단, Phase 2 튜닝을 돕는 4번 화면만
최소 형태로 Phase 2에 선행 투입하는 것은 허용한다(반나절).

---

## 2. 그래프 스키마

### 노드 (기본 13종 + 스택별 5종 = 18종)

| 타입 | 설명 |
|---|---|
| `Solution` | 솔루션 단위 (저장소 묶음) |
| `Repo` / `File` / `Module` | 저장소·파일·패키지 경계 |
| `Symbol` | 함수/클래스/메서드/타입/상수 (kind, signature, span, doc) |
| `Test` | 테스트 케이스 |
| `Doc` | md/ADR 섹션 |
| `Commit` / `Author` | git 이력 |
| `ExternalDep` | 외부 패키지 |
| `ConfigKey` | env/설정 키 (`application.yml` 포함) |
| `Contract` | OpenAPI/스키마/공유 타입 |
| **`Route`** | `@GetMapping` 등 / react-router (3.9절) |
| **`ApiCall`** | `fetch`·`axios` 호출 지점 (3.9절) |
| **`Bean`** | `@Component`/`@Service`/`@Repository` (3.9절) |
| **`Entity` · `Table`** | JPA 엔티티, `@Table`, 마이그레이션 SQL (3.9절) |
| **`Control`** | WinForms 컨트롤 선언 (3.9절) |

### 엣지 (기본 14종 + 스택별 5종 = 19종)

`CONTAINS`, `DEFINED_IN`, `IMPORTS`, `CALLS`, `REFERENCES`, `EXTENDS_IMPLEMENTS`,
`TESTS`, `DOCUMENTS`, `MODIFIED_BY`, `AUTHORED_BY`, `CO_CHANGED_WITH`(weight),
`DEPENDS_ON`, `EXPOSES`, `SHARES_CONTRACT`

스택별 추가: **`CALLS_API`**(ApiCall→Route ★교차 저장소 핵심), `INJECTS`(Bean→Bean),
`PERSISTS_TO`(Repository/Entity→Table), `HANDLES`(Control 이벤트→핸들러), `DUPLICATE_OF`

모든 엣지는 **provenance(`fast`/`precise`)와 confidence**를 속성으로 갖는다(3.9절).

### 저장소 선택 — 임베디드 그래프 DB 조사 결과

"가벼우면서 로컬에서 쓸 수 있는 그래프 DB"는 **존재한다.**
Kuzu 아카이브(2025-10, Apple 인수) 이후 후계 생태계가 형성됐다.

#### 임베디드(서버 불필요) 후보

| DB | 질의어 | 바인딩 | 라이선스 | 상태 | 특징 |
|---|---|---|---|---|---|
| **LadybugDB** | Cypher | **Node.js / Python / Rust** | **MIT** | Kuzu 포크, 활발 | 컬럼나 저장, DuckDB 상호운용, Arrow/Parquet, 멀티라벨, 서브그래프 격리. `brew install ladybug` |
| **FalkorDBLite** | Cypher | Python 중심 | — | 개발 중 | **DB 하나에 그래프 여러 개** — 다중 솔루션 격리에 유리 |
| **Raphtory** | 자체 | Rust/Python | — | 성숙, 활발 | 시간(temporal) 그래프가 1급 |
| **TuringDB** | — | — | — | 개발 중 | git 유사 버저닝(commit/branch/merge/time-travel) |
| **Lance Graph** | — | — | — | LanceDB 내부 | Lance 컬럼나 포맷 위의 그래프 |
| **HelixDB** | — | Rust | OSS | 신생 | 그래프+벡터 통합 |
| ~~Kuzu~~ | Cypher | — | — | **아카이브(2025-10)** | 채택 불가 |

서버형(참고): Neo4j, FalkorDB, Memgraph, ArcadeDB, ArangoDB.

#### 우리 워크로드와의 불일치 — 두 가지 실질적 걸림돌

**(1) 워크로드 성격.** LadybugDB/Kuzu 계열은 **컬럼나 + 분석 질의 지향**이며
공식 소개도 "실시간 트랜잭션이 아닌 분석 워크로드"를 표방한다. 우리는 반대다:

| 우리 워크로드 | 성격 |
|---|---|
| 파일 저장 시 증분 재인덱싱 | **작고 잦은 쓰기** (OLTP형) |
| 에이전트 질의 (`nunchi_find`/`nunchi_neighbors`) | 소규모 포인트 질의, **저지연(<50ms)** 다수 |
| `nunchi_pack` 랭킹 | 서브그래프 PPR — 메모리 상주로 해결 |

**(2) 단일 라이터 제약.** 임베디드 DB는 한 프로세스·한 파일이며 두 프로세스가
동시에 쓰기로 열 수 없다. 우리는 인덱서(쓰기)와 MCP 서버(읽기)가 별도 프로세스다.
SQLite는 **WAL 모드로 쓰기 중 동시 읽기가 기본 지원**되어 이 문제가 없다.

#### 결정

> **v1: SQLite(WAL) + FTS5 + 메모리 상주 인접리스트.**
> **LadybugDB는 어댑터 뒤의 1급 후보로 두고, 아래 스파이크 결과로 전환 판단.**

근거는 "SQLite가 더 좋은 DB라서"가 아니라 우리 워크로드가 **잦은 소량 쓰기 +
저지연 포인트 질의**이고, 순회·PPR은 어차피 메모리에서 처리하기 때문이다
(엣지 100만 ≈ 50MB). 이 구조에서 그래프 DB의 순회 이점이 상쇄된다.

**전환 판단용 스파이크 (2시간, Phase 1에 포함)** — 추측하지 말고 잰다.

1. 초기 벌크 적재 시간
2. **파일 50개 증분 갱신 시간** ← 결정적 지표
3. 포인트 질의 1,000회 p50/p95 지연
4. 3홉 경로 질의 지연
5. 디스크·메모리 사용량
6. 프로세스 분리(쓰기1 + 읽기1) 동작 가능 여부

LadybugDB가 2·3번에서 밀리지 않으면 **Cypher 표현력·시각화·멀티라벨 이점이
크므로 채택**한다. 밀리면 SQLite로 간다.

**LadybugDB 채택 시 리스크(명시)**: 아카이브된 프로젝트의 포크이며 소규모 팀
스튜어드십. 부모 프로젝트가 이미 한 번 죽었다. 어댑터 6개 메서드
(`upsertNodes/upsertEdges/neighbors/paths/search/rank`)를 반드시 먼저 세워
교체 비용을 하루 이내로 묶는 것이 대응책이다.

**서버 그래프 DB로 전환할 트리거**
- 인덱스를 팀이 공유하는 서비스로 운영해야 할 때
- 엣지가 5천만을 넘어 메모리 상주가 불가능해질 때
- 4홉 이상 임의 경로 탐색이 상시 질의가 될 때
- 그래프 자체를 사람이 시각적으로 탐색하는 것이 주 사용처가 될 때

---

## 3. 질의 인터페이스 (MCP 툴 5개)

| 툴 | 입력 | 반환 (토큰 목표) |
|---|---|---|
| `nunchi_find` | 자연어/식별자 질의 | 상위 N개 심볼·파일·문서 + `path:line` + 1줄 요약 (~300t) |
| `nunchi_neighbors` | node, edge_types, depth | 호출자/피호출자/구현체/테스트 (~200t) |
| `nunchi_impact` | node | 전이 참조 + 동시변경 + 관련 테스트 (~400t) |
| `nunchi_pack` | task 설명, token_budget | **컨텍스트 팩**: 랭킹된 코드 스켈레톤 (기본 4k) |
| `nunchi_recent` | path/symbol | 최근 커밋·PR·결정 이력 (~300t) |

동일 기능을 CLI로도 제공: `nunchi find …`, `nunchi pack --budget 4000 "…"` (JSON 출력).

### `nunchi_pack` — 토큰 절감의 본체

```
score(n) = α·BM25(query, n.text)
         + β·PersonalizedPageRank(seeds=질의매치노드)
         + γ·recency(n)
         + δ·cochange(n, seeds)
         + ε·centrality(n)
```

렌더링 티어(예산 소진 시 자동 강등):
- **L0**: 시그니처 1줄 + `path:line`
- **L1**: 시그니처 + docstring + 핵심 5~15줄
- **L2**: 전체 본문 (상위 2~3개만)

"파일 12개 전체(≈35k)" 대신 "심볼 40개 L0/L1 혼합(≈4k) + 정확한 좌표"를 준다.

---

## 3.5 동작 방식 — MCP 관점과 TUI 관점

### 프로세스 구조

```
  ┌─────────────────────────────────────────────────┐
  │  nunchi index --watch      (데몬 · 쓰기 소유 1개)     │
  │  파일 변경 → 해시 비교 → 변경분만 재추출 → 커밋    │
  └────────────────────┬────────────────────────────┘
                       │ 쓰기
                       ▼
              ┌──────────────────┐
              │  graph.db        │  SQLite(WAL)
              └────────┬─────────┘
             읽기 ┌────┴────┐ 읽기
                  ▼         ▼
      ┌────────────────┐  ┌────────────────┐
      │ nunchi serve       │  │ nunchi tui         │
      │ MCP · stdio    │  │ ratatui        │
      │ 소비자: 에이전트│  │ 소비자: 사람    │
      └───────┬────────┘  └───────┬────────┘
              └────────┬──────────┘
                       ▼
             ┌──────────────────────────────┐
             │  core (공용 라이브러리)        │
             │  store / graph / rank / pack │
             └──────────────────────────────┘
```

**핵심**: TUI는 MCP 서버의 클라이언트가 아니다. 둘 다 `core`를 직접 호출한다.
→ 로직 중복이 없고, **TUI에 보이는 것이 에이전트가 받는 것과 동일**하다.

### MCP 관점 — 실제 트레이스

시나리오: *"주문 조회 실패 시 재시도 로직 고쳐줘"*

**지금 (그래프 없음)**
```
1. Grep "retry"                          → 40 hits
2. Read OrderService.java, RetryConfig.java …   → 18k tokens
3. Grep "order"                          → 60 hits
4. Read 4 files                          → 22k tokens
   ⋮
   12 turns · 입력 ~60k tokens · 프런트 영향은 끝내 모름
```

**그래프 도입 후**
```
1. nunchi_pack{task:"주문 조회 재시도 로직 수정", budget:4000}   → 3.8k tokens
2. Read OrderService.java:88-141 (팩이 지목한 범위만)         → 3k
3. Edit
   3 turns · 입력 ~9k tokens · React 훅 영향까지 인지
```

**`nunchi_pack` 반환 형태**

```jsonc
{
  "budget": 4000, "used": 3820,
  "seeds": ["OrderService.findOne", "RetryPolicy"],
  "items": [
    { "tier": "L2",
      "ref": "api/src/main/java/.../OrderService.java:88-141",
      "sym": "OrderService.findOne",
      "why": { "bm25": 0.81, "ppr": 0.63, "cochange": 0.44 },
      "body": "..." },
    { "tier": "L1",
      "ref": "api/src/main/java/.../OrderController.java:44",
      "sym": "OrderController.getOrder",
      "sig": "@GetMapping(\"/api/orders/{id}\") ResponseEntity<OrderDto> getOrder(...)",
      "calls": ["OrderService.findOne"] },
    { "tier": "L0",
      "ref": "api/src/test/java/.../OrderServiceTest.java:12",
      "sym": "findOne_retriesOnTimeout" }
  ],
  "related": {
    "cochanged": ["api/src/main/java/.../OrderRepository.java"],
    "cross_repo": [
      { "repo": "react-web", "sym": "useOrder",
        "ref": "src/hooks/useOrder.ts:31", "via": "CALLS_API" }
    ]
  },
  "stale": []
}
```

`cross_repo` 항목이 이 프로젝트의 존재 이유다.
**"이거 고치면 프런트의 저 훅이 깨진다"는 어떤 grep으로도 나오지 않는다.**

**`nunchi_pack` 내부 파이프라인 5단계**

| 단계 | 처리 | 자원 |
|---|---|---|
| 1. 시드 | 태스크 문장 → FTS5 BM25 → 상위 k개 노드 | 디스크 |
| 2. 확장 | 시드 기준 PPR, depth ≤ 3 | **메모리 인접리스트** |
| 3. 보강 | 동시변경·테스트·문서·`CALLS_API` 엣지 추가 | 메모리 |
| 4. 랭킹 | α~ε 가중 합산 | `nunchi.toml` |
| 5. 예산 렌더링 | 점수순 greedy, 예산 소진 시 **L2→L1→L0 강등** | — |

에이전트가 받는 것은 **답이 아니라 좌표**다. 이 "선별 + 강등"이 60k를 4k로 만든다.

### TUI 관점 — 같은 코어, 사람용 창

```
┌─ nunchi · Pack Preview ─────────────────────── graph.db · fresh · 2m ago ─┐
│ task: 주문 조회 재시도 로직 수정                       budget: 4000  │
├───────────────────────────────────────────────────────────────────────┤
│  tier  tok   score  symbol                  ref                       │
│  L2    1420  0.81   OrderService.findOne    OrderService.java:88-141  │
│  L1     680  0.74   OrderController.getOrder OrderController.java:44  │
│  L1     540  0.61   OrderRepository.findById OrderRepository.java:19  │
│  L0      90  0.38   findOne_retriesOnTimeout OrderServiceTest.java:12 │
│  ─────────────────────────────────────────────────────────────        │
│  ✦ cross-repo  react-web :: useOrder  useOrder.ts:31   (CALLS_API)    │
├───────────────────────────────────────────────────────────────────────┤
│ α bm25    ▓▓▓▓▓▓▓░░░ 0.70    δ cochange ▓▓▓▓░░░░░░ 0.40               │
│ β ppr     ▓▓▓▓▓░░░░░ 0.50    ε central  ▓▓░░░░░░░░ 0.20               │
│ γ recency ▓▓▓░░░░░░░ 0.30                     used 3820/4000  (95%)   │
└─ [tab] 화면  [←→] 가중치  [s] nunchi.toml 저장  [enter] 항목 열기 ────────┘
```

슬라이더를 움직이면 **즉시 재랭킹**된다. `s`로 저장하면 그 순간부터 에이전트도
같은 가중치를 쓴다. 랭킹 튜닝이 감이 아니라 관찰이 된다.

| 화면 | 하는 일 | 잡아내는 문제 |
|---|---|---|
| ① 탐색 | 심볼 검색 → 이웃 드릴다운 | **추출 오류** — 호출 엣지가 아예 없음 |
| ② 영향 범위 | `nunchi_impact`를 트리뷰로 | 영향 분석 누락 |
| ③ 인덱스 상태 | 진행률, **언어별 커버리지**, stale 비율 | "Java 파일 800개가 파싱 실패 중" |
| ④ 팩 미리보기 | 위 화면 | **랭킹 문제** |
| ⑤ 벤치 | grounded vs ungrounded 비교 | 회귀 |

**TUI가 존재해야 하는 진짜 이유**: 에이전트가 헛다리를 짚었을 때 원인이
*추출 실패*인지 *랭킹 오류*인지 *인덱스 노후*인지 사람이 갈라낼 수단이 필요하다.

---

## 3.6 인덱스 신선도 — 재인덱싱 전략

코드가 바뀔 때마다 재인덱싱하는가? → **변경분만. 전체는 거의 하지 않는다.**
설계의 승패는 "무엇을 쓰기 경로에 두느냐"에서 갈린다.

### 갱신 비용의 3계층 — 섞으면 안 된다

| 계층 | 대상 | 파일 1개 변경 시 | 처리 시점 |
|---|---|---|---|
| **A. 파일 내부** | 심볼, 시그니처, 파일 내 엣지 | **<1ms** (tree-sitter) | 즉시 |
| **B. 파일 간 엣지** | 이 파일을 참조하던 다른 파일 | 역인덱스로 영향 파일만 → 수십 개 | 즉시 |
| **C. 전역 파생값** | PageRank, 임베딩, 동시변경 통계 | **비싸다 — 전체 스캔** | **절대 즉시 아님** |

> **핵심 설계 결정: C를 쓰기 경로에서 완전히 제거한다.**
> - **PPR/근접도** → 저장하지 않는다. 시드 의존적이라 미리 계산 불가. 질의 시점 메모리 계산
> - **전역 PageRank** → 하루 1회 또는 유휴 시 배치. 하루 낡아도 랭킹에 무해
> - **임베딩** → 백그라운드 큐. 없으면 BM25 폴백(graceful degradation)
> - **동시변경 통계** → git 이력 기반이므로 **커밋 시점에만** 갱신. 파일 저장과 무관
>
> 이 분리를 안 하면 파일 저장마다 수 초씩 멈춘다. 대부분의 인덱서가 여기서 무너진다.

### 트리거 3종 — 셋 다 필요하다

**① 파일 워처** (`nunchi index --watch`) — 저장 즉시, debounce 300~500ms. A+B만 처리.

**② git 훅** — `post-commit`(C의 동시변경 통계) / `post-checkout` / `post-merge`.

**③ 질의 시점 지연 검증(lazy) ★ 안전망 — 없으면 시스템이 틀린 답을 준다**

인덱스는 **반드시 낡는다**. 무엇보다 **에이전트 자신이 방금 코드를 고쳤기 때문이다.**
Edit 직후 워처가 반응하기 전에 다음 `nunchi_pack`이 들어오는 레이스는 일상이다.

```
1. mtime + size 비교      ← 싸다. 대부분 여기서 통과
2. 불일치 시에만 해시 계산
3. 해시도 불일치 → 둘 중 하나:
   a) 그 파일만 즉석 재파싱 (<1ms) 후 진행        ← 기본
   b) 응답의 "stale": [...] 에 표시하고 반환       ← 재파싱 실패 시
```

**틀린 좌표를 자신 있게 주는 것보다 낡았다고 말하는 편이 항상 낫다.**

### 전체 재인덱싱이 필요한 경우 (드물다)

- **추출기/스키마 버전 변경** — `schema_version` 불일치 시 자동 전체 재빌드
- 인덱스 손상 / 새 언어 지원 추가

### 성능 목표와 검증

| 항목 | 목표 |
|---|---|
| 파일 1개 저장 → 반영 | < 50ms |
| 파일 50개 배치 | < 500ms ← Phase 1 스파이크의 **결정적 지표** |
| 브랜치 전환(1,000 파일) | < 5s |
| 콜드 전체 인덱싱 | 파일 1만 기준 수 초~수십 초 |

> 위 목표치는 **추정이며 실측 전이다.** 특히 컬럼나 엔진(LadybugDB)은 잦은 소량
> 쓰기가 약점이라 "파일 50개 배치" 수치가 엔진 선택을 가를 것이다(2절).

### 쓰기가 읽기를 막지 않아야 한다

- **SQLite**: WAL 모드로 쓰기 중 동시 읽기 기본 보장 → 문제 없음
- **LadybugDB**: 단일 라이터 제약 확인 필요 → 스파이크 항목 6번

---

## 3.7 브랜치 전환 — 콘텐츠 주소 캐시로 해결

### 문제 정의

인덱스는 **워킹트리 한 상태의 투영**이다. `git checkout`이 그 상태를 갈아치우면
인덱스도 갈아엎힌다. 그리고 실무에서 이건 왕복이다.

```
main ──checkout──▶ feature ──checkout──▶ main
     1,200 파일 재인덱싱      1,200 파일 재인덱싱
     (새로운 것 없음. 아까 이미 파싱한 내용이다)
```

### 해결 — A 계층은 브랜치의 함수가 아니라 **내용의 함수**다

```
parse(blob_content) → { symbols, edges, spans }     ← 브랜치와 무관
```

브랜치가 아니라 **내용 해시로 캐싱**한다. git이 blob을 다루는 방식 그대로다.

```
┌──────────────────────────────────────────────┐
│  extract_cache   (브랜치 무관 · 저장소 공용)   │
│  blob_sha256 → 직렬화된 추출 사실             │
└──────────────────────────────────────────────┘
             ▲ 조회                  ▲
      ┌──────┴──────┐         ┌──────┴──────┐
      │ main 인덱스 │         │ feature 인덱스│   ← 활성 blob 집합만 다름
      └─────────────┘         └─────────────┘
```

| 전환 | 캐시 없음 | 캐시 있음 |
|---|---|---|
| main → feature (최초) | 1,200 파싱 | 1,200 파싱 (캐시 채움) |
| feature → main (복귀) | 1,200 파싱 | **파싱 0회 · 캐시 히트 100%** |
| 이후 왕복 | 매번 1,200 | 매번 0 |

비용은 디스크뿐이며 LRU로 관리한다(예: 상한 2GB, 미사용 30일 축출).

### 브랜치 무관한 것 / 브랜치 의존적인 것

| 대상 | checkout 영향 | 이유 |
|---|---|---|
| A. 파일 내부 사실 | **없음** (캐시 재사용) | 내용의 함수 |
| B. 파일 간 심볼 해소 | **있음** | 같은 이름이 다른 파일을 가리킬 수 있다 |
| C. 동시변경 통계 | **없음** | git 이력 기반 |
| C. 전역 PageRank | 있음 (그러나 지연 처리) | 3.6절대로 배치 |

즉 checkout이 실제로 강제하는 재계산은 **B 계층 하나**다.

### 짧은 전환은 아예 인덱싱하지 않는다

```
post-checkout 훅  →  인덱스를 dirty로 표시 + 변경 파일 목록만 기록 (수 ms)
                  →  2초 debounce. 그 사이 또 checkout이면 취소하고 목록 병합
                  →  유휴 시 배치 해소
                  →  그 전에 nunchi_pack 질의가 들어오면
                     **해당 질의에 필요한 파일만 즉석 해소** 후 응답
```

**checkout 자체는 거의 공짜여야 한다.**

### 워크트리 — 병렬 작업의 정답

```
repo/.nunchi/
├── cache/          ← blob 추출 캐시 (전 워크트리 공유)
├── main.db
└── wt-feature.db
```

### 하지 않을 것 (v1)

- **브랜치별 전체 인덱스 복제** — 위 캐시 방식의 하위 호환이다.
- **비트템포럴 다중 버전 그래프** — `valid_from`/`valid_to`로 여러 브랜치를 한
  그래프에 공존(선례: `Phoenixrr2113/codebase-graph`, TuringDB). 강력하지만
  **모든 질의에 버전 필터가 붙어 복잡도가 계단식으로 뛴다.**
  → 전환 트리거: "여러 브랜치를 **동시에** 질의해야 한다"는 요구가 생길 때.
  콘텐츠 주소 캐시는 이 방향의 디딤돌이므로 지금 선택이 막다른 길은 아니다.

---

## 3.8 최초 적용(온보딩) — CLI가 경로다

### 원칙: TUI는 온보딩 도구가 아니다

| | CLI | TUI |
|---|---|---|
| 솔루션 N개에 반복 적용 | ✅ 스크립트 | ❌ 사람이 앉아야 함 |
| CI·헤드리스·원격 | ✅ | ❌ |
| 에이전트가 직접 실행 | ✅ | ❌ |
| "왜 결과가 이상한가" 진단 | 가능(텍스트) | ✅ 편함 |

> **설계 규칙: TUI는 CLI가 이미 내놓는 데이터의 뷰어일 뿐, 고유 기능을 갖지 않는다.**

### 온보딩 4단계

```bash
nunchi init          # ① nunchi.toml 생성 — 저장소·언어 감지, 제외 패턴 제안
nunchi index         # ② 최초 전체 인덱싱 (진행률 출력)
nunchi doctor        # ③ 품질 검증 ★ 여기가 진짜 관문
nunchi serve         # ④ MCP 등록 후 사용 시작
```

### ① `nunchi init`

```toml
[solution]
name = "web"
repos = ["~/dev/order-api", "~/dev/order-web"]   # 다중 저장소 = 1 솔루션

[index]
languages = ["java", "typescript"]
exclude = ["**/node_modules/**", "**/build/**", "**/target/**",
           "**/dist/**", "**/generated/**", "**/*.min.js"]

[rank]                                   # 재컴파일 없이 튜닝 (1.6절)
alpha_bm25 = 0.7
beta_ppr = 0.5
gamma_recency = 0.3
delta_cochange = 0.4
epsilon_central = 0.2
```

**제외 패턴이 온보딩 품질의 절반이다.** 생성 코드·벤더 디렉터리가 들어오면
랭킹이 오염되고 `nunchi_pack`이 쓰레기를 반환한다.

### ③ `nunchi doctor` — 최초 적용의 진짜 관문

```
$ nunchi doctor
solution: web                                      2 repos · 6,140 files

언어 커버리지
  java          3,201 files   3,198 파싱   99.9%  ✓
  typescript    2,102 files   2,089 파싱   99.4%  ✓
  (기타)          837 files       —

심볼 해소율                    88.2%   ⚠  목표 95%
  미해소 참조 상위:
    외부 라이브러리 (spring, react)   2,140   ← 정상. ExternalDep로 처리됨
    DI 주입 지점 미해소                 612   ← @Autowired 인터페이스→구현 미연결
    동적 import (TS)                    204

인덱스     412 MB · 노드 284k · 엣지 1.1M · 소요 47s
제외 경고  build/generated 아래 1,203 파일이 인덱싱됨 — exclude 추가 권장

스모크 테스트                  4/5 통과
  ✗ "주문 조회" → OrderService 미발견 (DI 미해소 영향)
```

**핵심 지표는 심볼 해소율**이다. 낮으면 `CALLS`/`INJECTS` 엣지가 비어
그래프가 사실상 파일 목록으로 전락한다. 90% 미만이면 사용 전에 원인을 잡는다.

`--json` 플래그로 같은 내용을 기계가 읽게 낸다(CI 게이트용).
TUI ③번 화면은 **이 JSON을 렌더링할 뿐이다.**

### 적용 순서 — 한꺼번에 하지 않는다

1. **가장 작은 저장소 하나**로 시작. 커버리지·해소율 문제를 여기서 다 겪는다
2. Phase 0 벤치를 돌려 **실제 절감치 확인**
3. 수치가 확인되면 나머지에 스크립트로 확산
4. 교차 저장소 엣지는 **2개 이상 인덱싱된 뒤** 의미가 생긴다

---

## 3.9 타깃 스택 — Spring Boot(Java) · React · C# WinForms

확정된 대상:
- **솔루션 A (v1)**: Spring Boot **Java** 백엔드 + React 프런트
- **솔루션 B (v2)**: C# WinForms 데스크톱 앱 — **별개 DB 사용**

### 스택별 함정과 대응

#### ① Spring Boot — 배선이 문법에 없다

가장 어려운 대상이다. 호출 관계가 **어노테이션과 DI로 구성**되어 소스에 구문적
호출이 존재하지 않는다.

| 실제 관계 | 소스에 보이는 것 | 필요한 처리 |
|---|---|---|
| `GET /api/orders` → 컨트롤러 메서드 | `@GetMapping("/api/orders")` | `Route` 노드 + `EXPOSES` 엣지 |
| 서비스 → 구현체 | `@Autowired OrderRepository` (인터페이스) | `INJECTS` 엣지 + 인터페이스→구현 해소 |
| `findByStatus(...)` | **메서드 본문이 아예 없음** (JPA 파생 쿼리) | 메서드명 파싱 → `Entity`/`Table` 참조 |
| `@Transactional`, AOP | 없음 | 속성으로 기록 |
| `@Value("${db.url}")` | 문자열 | `ConfigKey` 노드 (`application.yml` 연결) |

처리하지 않으면 그래프에 **호출 엣지가 거의 생기지 않아 파일 목록으로 전락한다**
(3.8절 심볼 해소율이 이걸 잡는다).

#### ② React — 훅과 API 호출

컴포넌트·훅은 `scip-typescript`로 정밀하게 잡힌다. 추가로 필요한 것은
**API 호출 지점 추출**(`fetch`/`axios`의 URL 리터럴 및 템플릿) — 교차 엣지의 한쪽 끝이다.

#### ③ C# WinForms (v2)

| 함정 | 대응 |
|---|---|
| `partial class Form1`이 `Form1.cs` + `Form1.Designer.cs`로 분리 | **부분 클래스 멤버를 하나의 Symbol 노드로 병합** (필수) |
| `Designer.cs`가 자동 생성 + 거대 | 통째 제외 금지. **컨트롤 선언과 이벤트 배선만 추출**, 레이아웃 보일러플레이트는 버림 |
| `btnSave.Click += btnSave_Click` | `Control` 노드 + `HANDLES` 엣지 |
| `.resx`, `.csproj`/`.sln` | 모듈 경계와 프로젝트 참조로 사용 |

---

### ★ v1의 최대 수확 — 교차 저장소 계약 엣지

```
  React                          Spring Boot (Java)              DB
  ─────                          ──────────────────              ──
  useOrder.ts:31                 OrderController.java:44
  fetch(`/api/orders/${id}`) ──▶ @GetMapping("/api/orders/{id}")
        │        CALLS_API              │ EXPOSES
        │                               ▼
        │                        OrderService.findOne
        │                               │
        │                               ▼
        │                        OrderRepository (JPA)
        │                               │ PERSISTS_TO
        │                               ▼
        │                          orders 테이블
        │                          (WinForms는 별개 DB — 합류하지 않음)
```

*"주문 조회 API 응답 필드를 바꾸면 무엇이 깨지나"* 라는 질문에
**프런트 훅·컴포넌트, 백엔드 서비스, 테이블까지** 한 번에 답한다.
이것이 기성품으로 대체 불가능한 지점이다.

URL 매칭은 경로 템플릿 정규화(`/api/orders/{id}` ≡ `/api/orders/${id}`)로 수행하고,
확신도(confidence)를 엣지 속성에 기록한다.

### 저장소 구성 — 모노레포 vs 분리 (양쪽 다 지원)

`CALLS_API` 엣지 **생성 자체는 저장소 구성과 무관하다.** 그래프는 Solution 단위로
묶이므로 두 트리가 같은 인덱스에 들어오기만 하면 URL 매칭은 동일하게 동작한다.
실제로 갈리는 것은 두 가지뿐이다.

**① 교차 동시변경 신호**

| | 모노레포 | 분리 저장소 |
|---|---|---|
| "API와 훅을 같이 고쳤다" | **커밋 하나로 관측** | 별개 타임라인 |
| `CO_CHANGED_WITH` 교차 엣지 | 공짜로 생성 | 이슈 키·시간 창·작성자 **상관 추론** 필요 |
| 랭킹 영향 | `CALLS_API`를 독립 검증하는 2차 신호 확보 | 신호 약함, 별도 구현 (+0.5일) |

**② 브랜치 정합성 — 분리 저장소에서만 발생하는 문제**

프런트가 `feature/x`, 백엔드가 `main`에 있으면 인덱스는 **실제로 존재하지 않는
조합**을 표현한다. 프런트의 새 `fetch`가 백엔드 main에 없는 라우트를 가리킬 때
그것이 *버그*인지 *미머지*인지 그래프는 구분할 수 없다.

> **대응 (분리 저장소인 경우 필수)**
> - 인덱스에 **저장소별 HEAD(브랜치·커밋)를 기록**한다
> - 미해소 `CALLS_API`를 "라우트 없음"으로 단정하지 않고
>   **`unresolved_route` + 저장소 버전 정보**를 함께 반환한다
> - `nunchi doctor`가 저장소 간 HEAD 편차를 경고한다
>   (예: `order-web@feature/x` vs `order-api@main` — 교차 엣지 신뢰도 하향)
> - 추가 비용 약 0.5~1일

**결론: 양쪽 다 지원하도록 설계하며, 어느 쪽이든 v1은 동작한다.**
분리 저장소인 경우에만 위 두 항목(+0.5~1일)이 붙는다. 블로킹 사항이 아니다.

### WinForms는 독립 섬이다 — 가치 재평가

WinForms 앱이 **별개 DB**를 쓰고 Spring Boot 백엔드도 공유하지 않으므로,
위 교차 계약 그래프에 **구조적으로 합류하지 않는다.**

| 연결 종류 | 가능 여부 |
|---|---|
| `CALLS_API` (같은 API 호출) | ❌ 없음 |
| `PERSISTS_TO` (같은 테이블) | ❌ 별개 DB |
| 도메인 개념 유사성 ("주문", "고객") | △ **시맨틱 유사도만** — Phase 4 임베딩 영역, 하드 엣지 아님 |

**그래도 WinForms 인덱싱에 남는 가치**(솔루션 내부 그라운딩)는 실재한다:

1. **오래된 LOB 앱일수록 에이전트가 헤맨다.** 문서가 없고 파일이 크며 명명이
   불규칙한 코드베이스가 정확히 그래프의 이득이 큰 곳이다
2. **자체 DB 스키마 그래프** — WinForms LOB 앱은 SQL이 문자열 리터럴이나
   저장 프로시저에 있는 경우가 많다. `SQL 리터럴 → Table` 추출로
   *"이 화면이 어떤 테이블을 건드리나"* 에 답한다. 이 앱에서는 최고가치 질의다
3. `HANDLES` 엣지로 *"이 버튼 누르면 무슨 일이 일어나나"* 에 답한다

> **결론: WinForms는 독립 솔루션으로 자체 인덱스를 갖되, v2로 미룬다.**
> 교차 저장소 가치가 없으므로 v1의 차별점(`CALLS_API`) 증명에 기여하지 않는다.
> (v1에 함께 넣기를 원하면 아래 일정에 별도 항목으로 분리해 두었다.)

---

### 추출기 전략 — 정밀도와 속도의 2단 구성

| 스택 | 정밀 인덱서 | 비고 |
|---|---|---|
| **Java** (Spring) ✔확정 | `scip-java` | Gradle/Maven 빌드 연동. Kotlin 추출기 불필요 — 범위 축소 |
| TypeScript/React | `scip-typescript` | 가장 성숙 |
| C# (v2) | `scip-dotnet` 또는 **Roslyn 헬퍼** | .NET SDK 보유 전제 시 Roslyn이 해소 정밀도 최상 |

**그런데 SCIP 인덱서는 빌드를 요구한다.** Gradle·MSBuild 빌드는 분 단위다.
파일 저장마다 돌릴 수 없다 → 3.6절의 "저장 시 <50ms" 목표와 정면 충돌한다.

> **해결: 2단 속도 인덱싱**
>
> | | 빠른 경로 | 정밀 경로 |
> |---|---|---|
> | 도구 | tree-sitter | SCIP 인덱서 |
> | 시점 | **파일 저장 시** (ms) | **커밋 / CI / 유휴 시** (분) |
> | 산출 | 심볼, 스팬, 파일 내 구조, 어노테이션·라우트 리터럴 | 정밀 크로스파일 참조 해소 |
> | 용도 | `nunchi_find` 정확도 유지 | `CALLS`/`INJECTS` 신뢰도 |
>
> 엣지에 **provenance(`fast`/`precise`)와 confidence를 기록**하고,
> `nunchi_pack`은 정밀 엣지를 우선한다. 정밀 인덱스가 낡으면 `nunchi doctor`가 경고한다.

이 2단 구조는 Java/C#처럼 빌드가 느린 생태계에서 **선택이 아니라 필수다.**

---

### 일정 재산정

**v1 — 웹 솔루션 (Spring Boot Java + React)**

| 항목 | 일수 |
|---|---|
| Phase 0 벤치 하네스 | 0.5 |
| Phase 1 코어 + 스토어 + tree-sitter (Java, TS, **Rust**=도그푸딩) | 4~5 |
| Phase 1b SCIP 정밀 경로 (`scip-java`, `scip-typescript`) + 2단 구성 | 2~3 |
| Phase 1c 프레임워크 의미론 (Spring 어노테이션·DI·JPA, React API 호출) | 3~4 |
| Phase 2 컨텍스트 패커 | 2 |
| Phase 3 이력·문서 + **`CALLS_API` 교차 계약 엣지** | 3 |
| Phase 3.5 TUI | 1~2 |
| Phase 4 시맨틱·NL | 1~2 |
| Phase 5 운영화 | 2 |
| **v1 합계** | **18~24일** |

**v2 — WinForms 추가 (별도)**

| 항목 | 일수 |
|---|---|
| C# 추출기 (tree-sitter + Roslyn 헬퍼) | 2~3 |
| partial 클래스 병합 · Designer 이벤트 배선 · `.resx` | 1~2 |
| SQL 리터럴/저장 프로시저 → `Table` 추출 | 1~2 |
| **v2 합계** | **4~7일** |

Kotlin 미사용 확정으로 이전 21~28일 추정에서 범위가 줄었다.

### 착수 순서 (권장)

1. **React 단독으로 Phase 0~2를 관통** — `scip-typescript`가 가장 성숙해 파이프라인을
   가장 빨리 완성할 수 있다. 여기서 토큰 절감 수치를 최초로 확보한다
2. **Spring Boot(Java) 추가** → 어노테이션·DI·JPA 의미론 구현
3. **`CALLS_API` 엣지 연결** → 교차 저장소 가치 실증. **v1의 하이라이트**
4. Phase 3.5~5로 운영화
5. **WinForms는 v2** — 독립 섬이므로 v1 차별점 증명에 기여하지 않는다

---

## 3.10 개발 환경 분리 — 회사 소스 없이 개발하기

실제 솔루션 소스는 **회사 내 컴퓨터에** 있고, 개발은 이 머신에서 한다.
이 제약은 해결 가능하며, 오히려 설계 결정 하나가 여기서 값을 한다.

### 저장소 경로가 실제로 필요한 시점

| | 경로 필요? | 이유 |
|---|---|---|
| 코어 (스토어·그래프·랭킹·패커) | ❌ | 도메인 무관 |
| MCP 서버 / CLI / TUI | ❌ | 도메인 무관 |
| 추출기 (tree-sitter Java·TS) | ❌ | **언어**의 함수이지 그들 코드의 함수가 아님 |
| 프레임워크 의미론 (Spring 어노테이션·DI·JPA, React fetch) | ❌ | **프레임워크**의 함수 |
| `CALLS_API` 교차 엣지 로직 | ❌ | URL 템플릿 정규화 규칙 |
| **Phase 0 벤치 (절감 실측)** | ✅ | 그들의 태스크·코드여야 의미 있음 |
| **`nunchi doctor` 커버리지·해소율** | ✅ | 그들의 코딩 컨벤션에 의존 |
| **랭킹 가중치 α~ε 튜닝** | ✅ | 그들의 코드 분포에 의존 |

> **결론: 도구 전체를 회사 코드 없이 개발할 수 있다.**
> 경로가 필요한 것은 **검증과 튜닝**이며, 그건 회사 컴에서 수행한다.

### 개발용 대리 저장소 — RealWorld(Conduit)

같은 스택의 공개 프로젝트로 개발·테스트한다. **RealWorld/Conduit**이 이 목적에 최적이다.

이유: RealWorld는 **하나의 API 명세**를 여러 언어로 구현한 프로젝트 모음이라,
Spring Boot 백엔드와 React 프런트가 **실제로 같은 계약을 공유한다.**
`CALLS_API` 교차 저장소 엣지를 검증할 수 있는 공개 테스트베드가 된다.

| 역할 | 후보 |
|---|---|
| Spring Boot 백엔드 | `gabrielgua/realworld-springboot` (Spring Boot 3 / Java 21, JPA·JWT), `sivaprasadreddy/spring-realworld-conduit-api`, `alexey-lapin/realworld-backend-spring` |
| React 프런트 | `romansndlr/react-vite-realworld-example-app` |

검증 시나리오 예: 프런트의 `GET /api/articles/:slug` 호출이 백엔드
`@GetMapping("/articles/{slug}")` 로 정확히 연결되는가 → `CALLS_API` 엣지 정답셋.

**보완**: Spring 관용구 다양성 확보를 위해 `spring-projects/spring-petclinic`를
추가 표본으로 둔다(전형적 JPA·레이어 구조).

### 회사 컴에서의 실행 모델

**1.6절에서 Rust 정적 바이너리를 고른 결정이 여기서 실질 이득이 된다.**
회사 컴에 Rust 툴체인·Node·Python 등 개발 환경을 구축할 필요가 없다.
바이너리 하나를 옮겨 실행한다.

```
[이 머신]  개발 · RealWorld로 테스트 · nunchi 바이너리 빌드
              │
              │  바이너리 1개 전달 (소스 아님)
              ▼
[회사 컴]  nunchi init → nunchi index → nunchi doctor --json
           인덱스(graph.db)는 회사 컴에 남는다 · 소스 반출 없음
              │
              │  nunchi doctor --json  (수치만: 커버리지·해소율·노드/엣지 수)
              │  nunchi bench --json   (토큰·턴·정답률 집계)
              ▼
[이 머신]  수치 기반으로 추출기 보완 · 가중치 조정 → 새 바이너리
```

**소스코드는 회사 밖으로 나오지 않는다.** 왕복하는 것은 통계 JSON뿐이다.
(`nunchi doctor --json`은 파일 경로·심볼명을 포함할 수 있으므로,
`--redact` 옵션으로 경로/식별자를 해시 처리하는 모드를 넣는다.)

**회사 컴 전제 조건**
- Java 빌드 도구(Gradle/Maven) — `scip-java` 정밀 경로에 필요. 이미 있을 것
- Node/npm — `scip-typescript`에 필요. 이미 있을 것
- **자체 빌드 바이너리 실행이 사내 정책상 허용되는가** ← 확인 필요
  - 불가하면 대안: 빠른 경로(tree-sitter)만으로 축소 운영하거나,
    회사 컴에서 소스로 빌드(Rust 툴체인 설치 필요)

### 양쪽 머신 모두에서 운영한다 (확정)

에이전트 작업을 로컬 컴과 회사 컴 **양쪽에서** 하므로, `nunchi`도 양쪽에 상주해야 한다.

| | 로컬 (macOS) | 회사 컴 |
|---|---|---|
| 대상 코드 | `nunchi` 자신, RealWorld 표본, 개인 프로젝트 | **실제 솔루션** (Spring Boot·React·WinForms) |
| 역할 | 개발 · 테스트 · 도그푸딩 | 실측 · 실사용 |
| 인덱스 | 로컬 `graph.db` | 회사 컴 `graph.db` |

**인덱스는 동기화하지 않는다.** 각 머신이 자기 코드만 인덱싱하므로 공유할 것이 없다.
설계가 단순해지는 지점이다.

#### 회사 컴 = Windows + Rust 툴체인 보유 (확정)

**빌드 전략 A 확정: 각 머신에서 네이티브 빌드.** 크로스 컴파일 불필요.
3.10절 최대 리스크가 해소됐다.

```
[도구 소스] git (private) ──▶ [로컬 macOS]  cargo build → nunchi
                          └─▶ [회사 Windows] cargo build → nunchi.exe
회사 코드는 이 흐름에 들어오지 않는다. 오가는 것은 도구 소스뿐이다.
```

부수 이득: v2 C# 정밀 경로(Roslyn/MSBuild)가 **Windows에서 네이티브로 동작**한다.
macOS였다면 이 경로가 사실상 막혔을 것이다.

**Windows 고유 대응 항목 (Phase 1/5에 반영)**

| 항목 | 문제 | 대응 |
|---|---|---|
| **경로 정규화** | 역슬래시 구분자 + NTFS 대소문자 비구분(보존형) | 노드 ID는 **슬래시 정규화 + 소문자 비교키**, 표시는 원본 보존 |
| **긴 경로(MAX_PATH 260)** | Spring의 깊은 패키지 + Gradle `build/` 조합에서 실제로 걸린다 | `\\?\` 접두 경로 사용 또는 긴 경로 지원 활성화. **Phase 1에서 즉시 검증** |
| **CRLF / `core.autocrlf`** | 워킹트리는 CRLF, git blob은 LF → 두 해시가 다르다 | 3.7절 `extract_cache` 키를 **git blob SHA가 아니라 워킹트리 파일 내용 해시**로 정의. 자기 일관성만 유지하면 된다 |
| **파일 워처** | `notify`가 `ReadDirectoryChangesW`로 흡수하나 대량 이벤트 동작이 다르다 | 브랜치 전환 이벤트 폭풍을 Windows에서 별도 검증 (3.7절 debounce) |
| **SQLite WAL** | 파일 잠금 의미가 POSIX와 다르다 | 인덱서(쓰기) + MCP(읽기) 동시 동작을 Windows에서 실측 |
| `scip-java` | Gradle/Maven Windows 실행 | 구현 시 확인 |

> 위 항목 중 **긴 경로**와 **파일 워처 이벤트 폭풍**이 실제로 물릴 가능성이 높다.
> 나머지는 정규화 규칙만 정하면 끝난다.

#### 머신별 설정 분리

경로가 머신마다 다르므로 `nunchi.toml`을 분리한다.

```
~/.config/nunchi/machines/local.toml     # 로컬 저장소 경로
~/.config/nunchi/machines/work.toml      # 회사 저장소 경로
repo/.nunchi/nunchi.toml                     # 저장소별 공통 설정 (제외 패턴, 랭킹 가중치)
```

**랭킹 가중치(α~ε)는 저장소 단위로 커밋**해 양쪽 머신이 같은 값을 쓰게 한다.

#### 도그푸딩 — `nunchi`가 자기 자신을 인덱싱한다

로컬에서 `nunchi`를 개발하면서 **`nunchi` 자신의 Rust 코드베이스를 인덱싱**한다.
`tree-sitter-rust`는 이미 네이티브라 추출기 추가 비용이 작고, 개발 내내
즉각적인 품질 피드백을 준다. 랭킹이 나쁘면 개발자 본인이 먼저 아프다.

→ Phase 1 언어 목록에 **Rust 추가** (Java, TypeScript, Rust)

### 이 구조가 개발 순서에 주는 영향

Phase 0(벤치 하네스)은 **회사 컴에서만 의미 있는 수치**를 낸다.
따라서 순서를 조정한다:

1. **여기서**: RealWorld로 Phase 1~2를 관통해 파이프라인 완성
   (벤치 하네스는 RealWorld 태스크로 먼저 검증 — 절대 수치가 아니라 *동작 확인*)
2. **회사 컴에서**: 실제 솔루션에 `nunchi index` + `nunchi doctor` → 커버리지·해소율 확보
3. **여기서**: 그 수치로 추출기 보완
4. **회사 컴에서**: Phase 0 벤치 실행 → **진짜 절감 수치 확보**
5. 2~4 반복

즉 Phase 0의 *구현*은 앞에 두되, *실측*은 회사 컴 접근 시점에 붙인다.

---

## 4. 단계별 실행 계획

### Phase 0 — 측정 기반 (0.5일) · **먼저 한다**

그래프를 만들기 전에 비교 기준이 없으면 "절감"을 주장할 수 없다.

- 실제 태스크 15~20개 수집 (버그 수정, 기능 추가, "X는 어디서 처리되나?" 류)
- 하네스: 동일 태스크를 `--no-graph` / `--graph`로 실행 → 입력 토큰, 턴 수,
  wall-clock, 정답 여부 기록
- 산출물: `bench/tasks.jsonl`, `bench/run.rs`, 베이스라인 리포트
- **실측은 회사 컴에서** — 여기서는 RealWorld 태스크로 동작 검증만 (3.10절)
- **동시에 즉시 적용할 공짜 레버** (그래프와 무관, 오늘 가능):
  - 사용하지 않는 MCP 서버 비활성화 / 지연 로딩 유지
  - 프롬프트 캐시 TTL 1시간 활용 패턴 정착
  - CLAUDE.md 다이어트 — 매 요청 재전송되는 상시 비용
  - 서브에이전트 기본 모델 하향

### Phase 1 — 코드 그래프 MVP (4~5일) · 자체 구현

- 스택: **Rust** (1.6절) — `tree-sitter` + `rusqlite` + `rmcp` + `ratatui`
- 산출물은 **단일 정적 바이너리** `nunchi` (서브커맨드: `init`/`index`/`doctor`/`serve`/`tui`)
- 스토어 어댑터 6개 메서드를 **가장 먼저** 정의 → 저장 계층 교체 가능하게 유지
- **엔진 스파이크 2시간**: SQLite vs LadybugDB 증분 쓰기·포인트 질의 실측 (2절)
- 언어: Java, TypeScript, **Rust**(자기 자신 인덱싱 = 도그푸딩, 3.10절)
- **Windows 선행 검증**: 긴 경로(MAX_PATH), 워처 이벤트 폭풍, WAL 동시 접근 (3.10절)
- 구현: `Repo/File/Module/Symbol` 노드 + `CONTAINS/DEFINED_IN/IMPORTS/CALLS/REFERENCES`
- 도구: `nunchi_find`, `nunchi_neighbors` + CLI
- 완료 기준: "이 기능 어디서 처리되나" 류 질문이 파일 Read 0~2회로 해결

### Phase 1b — SCIP 정밀 경로 (2~3일)

- `scip-java`, `scip-typescript` 연동, 2단 속도 인덱싱 구성 (3.9절)
- 엣지 provenance/confidence 기록

### Phase 1c — 프레임워크 의미론 (3~4일)

- Spring: `@RequestMapping` 계열 → `Route`/`EXPOSES`, DI → `INJECTS`,
  JPA → `Entity`/`Table`/`PERSISTS_TO`, `@Value` → `ConfigKey`
- React: `fetch`/`axios` 호출 지점 → `ApiCall`

### Phase 2 — 컨텍스트 패커 (2일) · **ROI 최대 구간**

- 랭킹 + 토큰 예산 렌더링(L0/L1/L2) 구현
- `nunchi_pack` 노출, Phase 0 하네스로 α~ε 튜닝
- 완료 기준: 벤치 태스크 평균 입력 토큰 **40% 이상** 감소, 정답률 비열화

### Phase 3 — 이력·문서·교차 계약 (3일)

- git 로그 → `Commit/Author/MODIFIED_BY/CO_CHANGED_WITH`
- md/ADR/README → `Doc/DOCUMENTS`
- **`CALLS_API` 교차 저장소 엣지** (URL 템플릿 정규화 + confidence)
- 도구: `nunchi_impact`, `nunchi_recent`
- 완료 기준: "이거 고치면 뭐가 깨지나"에 테스트·호출자·동시변경·**프런트 훅**까지 응답

### Phase 3.5 — TUI (1~2일) · 임계 경로 아님

- `ratatui` 5개 화면 (1.6절). 스토어 어댑터 위의 읽기 전용 뷰어
- 팩 미리보기 화면의 최소판은 Phase 2에 선행 투입 (반나절)
- 완료 기준: 그래프 품질 문제를 코드 수정 없이 TUI에서 진단 가능

### Phase 4 — 자연어 질의 & 시맨틱 (1~2일)

- 벡터 임베딩(`sqlite-vec` 또는 내장 인덱스)으로 어휘 미스 보완
- NL → 구조화 질의 변환기
- 완료 기준: 도메인 용어로 물어도 정확한 심볼로 착지

### Phase 5 — 운영화 (2일)

- 3.6절 전략 구현: 워처 + git 훅 3종 + **질의 시점 지연 검증(안전망)**
- 3.7절 구현: `extract_cache`(blob 해시 → 추출 사실, LRU), dirty 마킹 + debounce,
  워크트리별 인덱스 + 캐시 공유
- 3.8절 온보딩 CLI: `nunchi init` / `nunchi doctor`(+`--json`)
- `schema_version` 기반 자동 전체 재빌드
- 각 저장소 CLAUDE.md에 규칙 추가: "탐색 전 `nunchi pack`을 먼저 호출"

**총 소요: v1(웹 솔루션) 18~24일 + v2(WinForms) 4~7일** — 3.9절 재산정.

---

## 5. 리스크와 대응

| 리스크 | 대응 |
|---|---|
| 인덱스가 낡아 잘못된 좌표 반환 → 오히려 정확도 하락 | 해시 검증, stale이면 제외하고 경고 (3.6절) |
| **Spring DI/JPA 미해소로 그래프가 파일 목록화** | `nunchi doctor` 심볼 해소율 게이트(95% 목표), SCIP 정밀 경로 필수화 |
| 그래프 조회가 파일 읽기보다 비싸지는 역전 | `nunchi_pack` 예산 상한 강제, 벤치 상시 감시 |
| MCP 툴 5개도 상시 스키마 비용 | CLI 경로 기본 안내, MCP는 얇게 유지 |
| SCIP 빌드가 느려 정밀 인덱스가 만성 지연 | 2단 구조 + confidence 표기, CI에서 야간 갱신 |
| **Windows 긴 경로(MAX_PATH) / 워처 이벤트 폭풍** | Phase 1에서 Windows 실측 선행. `\\?\` 경로 + debounce 튜닝 (3.10절) |
| LadybugDB 채택 시 지속성 리스크 | 어댑터 6개 메서드로 교체 비용 1일 이내 고정 |
| 에이전트가 그래프를 안 쓰고 습관대로 grep | CLAUDE.md 규칙 + 벤치에서 사용률 측정 |

---

## 6. 필요한 입력 (사용자 확인 필요)

- [x] 대상 스택 — **Spring Boot(Java) + React** (v1) / **C# WinForms** (v2)
- [x] 구현 언어 — **Rust 확정** (1.6절)
- [x] WinForms는 **별개 DB** → 교차 그래프에서 독립 섬. v2로 후순위 (3.9절)

남은 확인 사항:

1. ~~저장소 경로~~ → **개발에는 불필요** (3.10절). 회사 컴 검증 단계에서만 필요
2. [x] 에이전트 작업 위치 — **양쪽 머신 모두** (3.10절 반영)
3. [x] 회사 컴 = **Windows**, Rust 툴체인 보유 → 빌드 전략 A 확정 (3.10절)
4. 저장소 구성(모노레포/분리) — 블로킹 아님. 분리면 3.9절 항목 +0.5~1일
5. 저장소별 대략 규모(파일 수)
6. Spring Boot 빌드 도구 — Gradle / Maven (구현 시점 확인도 무방)
7. Rust 숙련도 — 낮다면 Phase 1 일정에 반영
8. WinForms를 v1에 포함할지, v2로 미룰지 (권장: v2 — 3.9절)
9. GitHub PR/이슈 연동을 Phase 3에 포함할지

> **착수 블로커 없음.** 위 항목은 모두 진행 중 확인 가능하다.

---

## 참고

- Running a Software Factory Efficiently at Uber Scale — https://www.uber.com/us/en/blog/efficient-software-factory/
- How Uber built an AI software factory — https://newsletter.port.io/p/how-uber-built-a-software-factory
- 코드 인텔리전스 도구 비교 — https://rywalker.com/research/code-intelligence-tools
- Codebase-Memory (tree-sitter→SQLite→MCP) — https://arxiv.org/html/2603.27277v1
- Cognee: 로컬 SQLite / 프로덕션 Neo4j·FalkorDB — https://www.cognee.ai/blog/guides/ai-coding-agent-persistent-codebase-memory
- 임베디드 그래프 DB 지형 (Kuzu 이후) — https://gdotv.com/blog/kuzu-legacy-embedded-graph-database-landscape/
- From Kuzu to Ladybug — https://thedataquarry.com/blog/from-kuzu-to-ladybug/
- LadybugDB — https://ladybugdb.com/
- 공식 Rust MCP SDK — https://github.com/modelcontextprotocol/rust-sdk
- rmcp Tier 1 승격 — https://www.digitalapplied.com/blog/mcp-sdk-conformance-tiers-what-tier-1-means

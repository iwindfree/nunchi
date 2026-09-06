// 프레임워크를 쓰지 않으므로 tauri.conf.json 의 withGlobalTauri 로 주입받는다.
const invoke = window.__TAURI__?.core?.invoke;

/** 지금 열린 솔루션의 개요. 인덱싱이 끝나면 다시 불러 갱신한다. */
let data = null;
/** 최근에 연 솔루션 목록. */
let recent = [];

function setSolutionLabel() {
  const label = document.getElementById("solution-label");
  label.textContent = data?.config ? data.config.solution : "솔루션 선택";
  label.title = "솔루션 바꾸기";
}

const el = (html) => {
  const t = document.createElement("template");
  t.innerHTML = html.trim();
  return t.content.firstElementChild;
};

const esc = (s) =>
  String(s).replace(/[&<>"]/g, (c) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]
  );

const num = (n) => Number(n ?? 0).toLocaleString("ko-KR");

// ── 개요 ────────────────────────────────────────────────
function overviewView() {
  if (!data || !data.config) {
    // 연 솔루션이 없으면 무엇을 열지 먼저 고르게 한다.
    return solutionsView();
  }

  const c = data.config;
  const repos = c.repos
    .map(
      (r) => `<div><code>${esc(r.path)}</code>${
        r.exists ? "" : ' <span class="bad">경로를 찾을 수 없습니다</span>'
      }</div>`
    )
    .join("");

  // 인덱스가 낡았으면 지표보다 먼저 보여 준다. 아래 숫자들이 지금의 코드에
  // 대한 것이 아니라는 뜻이기 때문이다.
  const drift = driftPanel();

  const index = data.index
    ? `<div class="panel">
         <h3>인덱스</h3>
         <div class="stats">
           <div class="stat"><div class="n">${num(data.index.nodes)}</div><div class="k">노드</div></div>
           <div class="stat"><div class="n">${num(data.index.edges)}</div><div class="k">엣지</div></div>
           ${statFromMetrics(data.index.metrics)}
         </div>
       </div>
       ${languageTable(data.index.metrics)}`
    : `<div class="empty">
         <strong>아직 인덱싱하지 않았습니다.</strong>
         왼쪽의 인덱싱 화면에서 시작할 수 있습니다.
       </div>`;

  return `
    <h2>개요</h2>
    <p class="lead">설정과 인덱스의 현재 상태입니다.</p>

    <div class="panel">
      <h3>솔루션</h3>
      <dl>
        <dt>이름</dt><dd>${esc(c.solution)}</dd>
        <dt>저장소</dt><dd>${repos}</dd>
        <dt>언어</dt><dd>${c.languages.map(esc).join(", ")}</dd>
        <dt>설정 파일</dt><dd><code>${esc(c.path)}</code></dd>
        <dt>프레임워크 규칙</dt><dd>${c.rule_count}개</dd>
      </dl>
      <div class="actions">
        <button class="action" id="edit-repos">저장소 변경</button>
      </div>
    </div>

    ${drift}
    ${index}`;
}

/** 인덱스와 실제 코드의 차이. 어긋나지 않았으면 조용히 지나간다. */
function driftPanel() {
  const d = data.drift;
  if (!d) return "";
  const behind = d.changed + d.added + d.removed;
  if (behind === 0) {
    return `<div class="panel">
      <h3>인덱스 상태</h3>
      <p class="ok">실제 코드와 일치합니다. 파일 ${num(d.indexed)}개를 ${d.took_ms}밀리초에 확인했습니다.</p>
    </div>`;
  }
  const counts = [
    d.changed ? `바뀐 파일 ${num(d.changed)}개` : null,
    d.added ? `새 파일 ${num(d.added)}개` : null,
    d.removed ? `사라진 파일 ${num(d.removed)}개` : null,
  ].filter(Boolean).join(", ");
  return `<div class="panel">
    <h3>인덱스 상태</h3>
    <p class="bad">인덱스가 실제 코드와 어긋나 있습니다. ${esc(counts)}.</p>
    <ul class="hits">${d.examples.map((e) => `<li class="hit"><code>${esc(e)}</code></li>`).join("")}</ul>
    <div class="actions">
      <button class="primary" id="go-index">인덱싱 화면으로</button>
    </div>
    <p class="note">아래 숫자들은 마지막으로 인덱싱한 시점의 코드에 대한 것입니다.
      탐색과 팩도 바뀐 파일의 좌표를 내놓지 않습니다.</p>
  </div>`;
}

/** 지표에서 눈에 띄는 값 몇 개를 골라 큰 숫자로 보여 준다. */
function statFromMetrics(m) {
  if (!m || typeof m !== "object") return "";
  const out = [];
  if (m.symbols != null)
    out.push(`<div class="stat"><div class="n">${num(m.symbols)}</div><div class="k">심볼</div></div>`);
  if (m.routes != null)
    out.push(`<div class="stat"><div class="n">${num(m.routes)}</div><div class="k">라우트</div></div>`);
  if (m.api_calls != null)
    out.push(`<div class="stat"><div class="n">${num(m.api_calls)}</div><div class="k">API 호출</div></div>`);
  if (m.api_calls_linked != null)
    out.push(
      `<div class="stat"><div class="n">${num(m.api_calls_linked)}</div><div class="k">라우트에 연결</div></div>`
    );
  return out.join("");
}

/** 언어별 파싱 성공률. 낮으면 추출기에 문제가 있다는 신호다. */
function languageTable(m) {
  const langs = m?.by_lang;
  if (!Array.isArray(langs) || langs.length === 0) return "";
  const rows = langs
    .map((e) => {
      const files = e.files ?? 0;
      const parsed = e.parsed ?? 0;
      const pct = files > 0 ? (parsed / files) * 100 : 0;
      const cls = pct >= 99 ? "ok" : pct >= 90 ? "warn" : "bad";
      return `<tr>
        <td>${esc(e.lang ?? "?")}</td>
        <td class="num">${num(files)}</td>
        <td class="num">${num(parsed)}</td>
        <td class="num ${cls}">${pct.toFixed(1)}%</td>
      </tr>`;
    })
    .join("");
  return `<div class="panel">
    <h3>언어 커버리지</h3>
    <table>
      <thead><tr><th>언어</th><th class="num">파일</th><th class="num">파싱</th><th class="num">성공률</th></tr></thead>
      <tbody>${rows}</tbody>
    </table>
  </div>`;
}


// ── 솔루션 선택 ──────────────────────────────────────────
let openError = null;

function solutionsView() {
  const items = recent.length
    ? `<ul class="repo-list">${recent
        .map((e) => {
          const dir = e.config_path.replace(/[/\\]nunchi\.toml$/, "");
          return `<li>
            <div style="flex:1">
              <div><strong>${esc(e.name)}</strong>${
                e.exists ? "" : ' <span class="bad">파일이 없습니다</span>'
              }</div>
              <div class="muted"><code>${esc(dir)}</code></div>
            </div>
            ${
              e.exists
                ? `<button class="action" data-open="${esc(e.config_path)}">열기</button>`
                : ""
            }
            <button class="remove" data-forget="${esc(e.config_path)}">목록에서 제거</button>
          </li>`;
        })
        .join("")}</ul>`
    : `<p class="muted">아직 연 솔루션이 없습니다.</p>`;

  return `
    <h2>솔루션</h2>
    <p class="lead">최근에 연 솔루션에서 고르거나 새로 만들 수 있습니다.</p>
    ${openError ? `<div class="error">${esc(openError)}</div>` : ""}
    <div class="panel">
      <h3>최근에 연 솔루션</h3>
      ${items}
    </div>
    <div class="actions">
      <button class="action" id="open-existing">기존 솔루션 열기</button>
      <button class="primary" id="new-solution">새 솔루션 만들기</button>
    </div>
    <p class="note">기존 솔루션을 열려면 <code>nunchi.toml</code>이 있는 폴더를 고르십시오.</p>`;
}

async function openSolution(configPath) {
  openError = null;
  try {
    data = await invoke("open_solution", { configPath });
    recent = await invoke("recent_list");
    setSolutionLabel();
    show("overview");
  } catch (e) {
    openError = String(e);
    recent = await invoke("recent_list");
    show("solutions");
  }
}

function bindSolutions() {
  document.querySelectorAll("[data-open]").forEach((b) =>
    b.addEventListener("click", () => openSolution(b.dataset.open))
  );
  document.querySelectorAll("[data-forget]").forEach((b) =>
    b.addEventListener("click", async () => {
      recent = await invoke("forget_solution", { configPath: b.dataset.forget });
      show("solutions");
    })
  );
  document.getElementById("open-existing")?.addEventListener("click", async () => {
    openError = null;
    try {
      const path = await invoke("open_folder");
      if (path) await openSolution(path);
    } catch (e) {
      openError = String(e);
      show("solutions");
    }
  });
  document.getElementById("new-solution")?.addEventListener("click", async () => {
    draft.repos = [];
    draft.name = "";
    draft.languages = null;
    draft.error = null;
    draft.overwrite = false;
    draft.dir = "";
    show("setup");
  });
}

// ── 초기 설정 마법사 ─────────────────────────────────────
/** 설정을 만들기 전까지 들고 있는 입력값. */
const draft = {
  dir: "",
  repos: [],
  name: "",
  languages: null,
  error: null,
  busy: false,
  /// 이미 있는 설정을 고치는 중인가. 그렇다면 덮어쓴다.
  overwrite: false,
};

async function pickRepo() {
  const path = await invoke("pick_folder");
  if (!path) return;
  if (draft.repos.includes(path)) return;
  draft.repos.push(path);
  draft.languages = null;
  await refreshLanguages();
  show("setup");
}

async function pickDir() {
  const path = await invoke("pick_folder");
  if (!path) return;
  draft.dir = path;
  show("setup");
}

function removeRepo(path) {
  draft.repos = draft.repos.filter((r) => r !== path);
  draft.languages = null;
  refreshLanguages().then(() => show("setup"));
}

/** 고른 저장소에 어떤 언어가 있는지 미리 확인한다. */
async function refreshLanguages() {
  if (draft.repos.length === 0) {
    draft.languages = null;
    return;
  }
  try {
    draft.languages = await invoke("detect_languages", { repos: draft.repos });
  } catch (e) {
    draft.languages = [];
    draft.error = String(e);
  }
}

async function createConfig() {
  draft.busy = true;
  draft.error = null;
  show("setup");
  try {
    data = await invoke("init_solution", {
      dir: draft.dir,
      repos: draft.repos,
      name: draft.name,
      force: draft.overwrite,
    });
    recent = await invoke("recent_list");
    draft.busy = false;
    setSolutionLabel();
    show("overview");
  } catch (e) {
    draft.error = String(e);
    draft.busy = false;
    show("setup");
  }
}

function setupView() {
  const repos = draft.repos.length
    ? `<ul class="repo-list">${draft.repos
        .map(
          (r) => `<li><code>${esc(r)}</code>
            <button class="remove" data-remove="${esc(r)}">제거</button></li>`
        )
        .join("")}</ul>`
    : `<p class="muted">아직 고른 저장소가 없습니다.</p>`;

  const langs =
    draft.languages === null
      ? ""
      : draft.languages.length
        ? `<div class="panel">
             <h3>감지된 언어</h3>
             <div class="tags">${draft.languages.map((l) => `<span class="tag">${esc(l)}</span>`).join("")}</div>
             <p class="note">파일이 세 개 이상인 언어만 나옵니다. 설정 파일에서 나중에 고칠 수 있습니다.</p>
           </div>`
        : `<div class="panel">
             <h3>감지된 언어</h3>
             <p class="muted">코드 파일을 찾지 못했습니다. 기본값(Java, TypeScript, Rust)이 들어갑니다.</p>
           </div>`;

  const ready = draft.repos.length > 0 && draft.dir && !draft.busy;

  return `
    <h2>${draft.overwrite ? "저장소 변경" : "시작하기"}</h2>
    <p class="lead">${
      draft.overwrite
        ? "저장소를 더하거나 빼면 설정 파일을 다시 씁니다."
        : "인덱싱할 저장소를 고르면 설정 파일을 만들어 드립니다."
    }</p>

    ${draft.error ? `<div class="error">${esc(draft.error)}</div>` : ""}

    <div class="panel">
      <h3>저장소</h3>
      ${repos}
      <button class="action" id="add-repo">저장소 추가</button>
      <p class="note">Spring 백엔드와 React 프런트엔드처럼 여러 저장소를 함께 넣으면
        저장소를 건너는 API 호출까지 이어집니다.</p>
    </div>

    ${langs}

    <div class="panel">
      <h3>설정</h3>
      <label class="field">
        <span>솔루션 이름</span>
        <input type="text" id="sol-name" value="${esc(draft.name)}"
               placeholder="비워 두면 첫 저장소의 폴더 이름을 씁니다" />
      </label>
      <label class="field">
        <span>설정 파일을 만들 위치</span>
        <div class="row">
          <input type="text" id="dir" value="${esc(draft.dir)}" readonly
                 placeholder="선택 버튼으로 폴더를 고르십시오" />
          <button class="action" id="pick-dir">${draft.dir ? "변경" : "선택"}</button>
        </div>
      </label>
      <p class="note">이 위치에 <code>nunchi.toml</code>과 <code>nunchi.shared.toml</code>이 생깁니다.
        앞의 것은 경로가 들어 있어 커밋하지 않고, 뒤의 것은 공유합니다.</p>
    </div>

    <div class="actions">
      <button class="primary" id="create" ${ready ? "" : "disabled"}>
        ${draft.busy ? "만드는 중입니다" : draft.overwrite ? "설정 덮어쓰기" : "설정 만들기"}
      </button>
      <button class="action" id="cancel">취소</button>
    </div>
    ${
      draft.overwrite
        ? `<p class="note">저장소를 바꾸면 기존 인덱스가 실제 코드와 어긋납니다.
             설정을 저장한 뒤 다시 인덱싱하십시오.</p>`
        : ""
    }`;
}

/** 마법사 화면의 버튼과 입력을 연결한다. */
function bindSetup() {
  document.getElementById("add-repo")?.addEventListener("click", pickRepo);
  document.getElementById("pick-dir")?.addEventListener("click", pickDir);
  document.getElementById("create")?.addEventListener("click", createConfig);
  document.getElementById("cancel")?.addEventListener("click", () => {
    draft.overwrite = false;
    draft.error = null;
    // 열린 솔루션이 있으면 개요로, 없으면 솔루션 목록으로 돌아간다.
    show(data ? "overview" : "solutions");
  });
  document.querySelectorAll("[data-remove]").forEach((b) =>
    b.addEventListener("click", () => removeRepo(b.dataset.remove))
  );
  const nameInput = document.getElementById("sol-name");
  if (nameInput) nameInput.addEventListener("input", (e) => (draft.name = e.target.value));
}

// ── 인덱싱 ───────────────────────────────────────────────
/** 인덱싱이 도는 동안의 상태. 이벤트를 받아 갱신한다. */
const job = { running: false, message: null, error: null, stats: null, rebuild: false };

function indexView() {
  if (!data || !data.config) return needSolution("인덱싱");

  const done = job.stats
    ? `<div class="panel">
         <h3>마지막 결과</h3>
         <div class="stats">
           <div class="stat"><div class="n">${num(job.stats.files_indexed)}</div><div class="k">파일</div></div>
           <div class="stat"><div class="n">${num(job.stats.nodes)}</div><div class="k">노드</div></div>
           <div class="stat"><div class="n">${num(job.stats.edges)}</div><div class="k">엣지</div></div>
           <div class="stat"><div class="n">${cacheRate(job.stats)}</div><div class="k">캐시 적중</div></div>
         </div>
       </div>`
    : "";

  const status = job.running
    ? `<div class="panel"><h3>진행 상황</h3><p>${esc(job.message ?? "시작하는 중입니다.")}</p></div>`
    : "";

  return `
    <h2>인덱싱</h2>
    <p class="lead">저장소를 훑어 그래프를 만듭니다.</p>
    ${job.error ? `<div class="error">${esc(job.error)}</div>` : ""}
    ${status}
    <div class="panel">
      <h3>실행</h3>
      <label class="row" style="gap:6px;margin-bottom:12px">
        <input type="checkbox" id="rebuild" ${job.rebuild ? "checked" : ""} />
        <span>인덱스를 지우고 처음부터 다시 만듭니다</span>
      </label>
      <button class="primary" id="run-index" ${job.running ? "disabled" : ""}>
        ${job.running ? "인덱싱 중입니다" : "인덱싱 시작"}
      </button>
      <p class="note">파싱 결과는 내용 해시로 캐시되므로 두 번째부터는 훨씬 빠릅니다.
        브랜치를 오갈 때도 다시 파싱하지 않습니다.</p>
    </div>
    ${done}`;
}

function cacheRate(s) {
  const total = (s.cache_hits ?? 0) + (s.cache_misses ?? 0);
  if (total === 0) return "0%";
  return Math.round(((s.cache_hits ?? 0) / total) * 100) + "%";
}

/** 진행 상황 이벤트를 사람이 읽을 문장으로 바꾼다. */
function describeProgress(p) {
  switch (p.stage) {
    case "repo_started":
      return `저장소 ${p.index}/${p.total} — ${p.repo}`;
    case "scanning":
      return `${p.repo} 를 훑는 중입니다. 파일 ${num(p.files)}개를 처리했습니다.`;
    case "resolving":
      return "파일 사이의 참조를 잇는 중입니다.";
    case "history":
      return "git 이력을 읽는 중입니다.";
    case "saving":
      return "데이터베이스에 쓰는 중입니다.";
    default:
      return "진행 중입니다.";
  }
}

async function runIndex() {
  job.running = true;
  job.error = null;
  job.message = null;
  job.stats = null;
  show("index");
  try {
    await invoke("start_index", { rebuild: job.rebuild });
  } catch (e) {
    job.running = false;
    job.error = String(e);
    show("index");
  }
}

function bindIndex() {
  document.getElementById("run-index")?.addEventListener("click", runIndex);
  document.getElementById("rebuild")?.addEventListener("change", (e) => {
    job.rebuild = e.target.checked;
  });
}

/** 인덱싱 진행과 완료 이벤트를 받는다. */
async function listenIndexEvents() {
  const listen = window.__TAURI__?.event?.listen;
  if (!listen) return;
  await listen("index-progress", (e) => {
    job.message = describeProgress(e.payload);
    // 인덱싱 화면을 보고 있을 때만 다시 그린다.
    if (document.querySelector('.nav[data-view="index"][aria-current]')) show("index");
  });
  await listen("index-done", async (e) => {
    job.running = false;
    if (e.payload.ok) {
      job.stats = e.payload.stats;
      // 인덱스가 새로 생겼으니 개요도 갱신한다.
      data = await invoke("overview");
    } else {
      job.error = e.payload.error;
    }
    show("index");
  });
}

// ── 화면을 열 수 없을 때 ─────────────────────────────────
const needSolution = (title) => `<h2>${title}</h2>
  <div class="empty">
    <strong>먼저 솔루션을 여십시오.</strong>
    왼쪽 아래의 솔루션 이름을 누르면 목록이 나옵니다.
  </div>`;

const needIndex = (title) => `<h2>${title}</h2>
  <div class="empty">
    <strong>아직 인덱싱하지 않았습니다.</strong>
    왼쪽의 인덱싱 화면에서 먼저 실행하십시오.
  </div>`;

// ── 탐색 ─────────────────────────────────────────────────
/** 탐색 화면의 상태. 화면을 옮겼다 돌아와도 결과가 남아 있게 들고 있는다. */
const explore = {
  query: "",
  hits: null,
  selected: null,
  neighbors: null,
  /// 직접 이어진 것부터 보여 준다. 2홉으로 올리면 허브 함수에서 수백 건이 나온다.
  depth: 1,
  busy: false,
  error: null,
};

/** 결과 한 줄. `pick`을 주면 눌러서 고를 수 있는 줄이 된다. */
function hitRow(h, pick, selected) {
  const score = h.score > 0 ? `<span class="score">${h.score.toFixed(2)}</span>` : "";
  const sig = h.signature ? `<div class="sig">${esc(h.signature)}</div>` : "";
  const attr = pick ? ` data-pick="${esc(h.id)}"` : "";
  return `<li class="hit${selected ? " on" : ""}"${attr}>
    <div class="hit-head">
      <span class="kind">${esc(h.kind)}</span>
      <strong>${esc(h.name)}</strong>
      <span class="muted">${esc(h.repo)}</span>
      ${score}
    </div>
    ${sig}
    ${h.reference ? `<div class="ref"><code>${esc(h.reference)}</code></div>` : ""}
  </li>`;
}

function exploreView() {
  if (!data || !data.config) return needSolution("탐색");
  if (!data.index) return needIndex("탐색");

  const results =
    explore.hits === null
      ? `<p class="muted">찾을 이름이나 문장을 넣고 Enter를 누르십시오.</p>`
      : explore.hits.length === 0
        ? `<p class="muted">맞는 심볼이 없습니다. 인덱스는 영어 식별자로 되어 있으므로,
             한국어로 찾으려면 설정 화면의 도메인 용어 사전에 등록해야 합니다.</p>`
        : `<ul class="hits">${explore.hits
            .map((h) => hitRow(h, true, explore.selected === h.id))
            .join("")}</ul>`;

  const linked =
    explore.neighbors === null
      ? ""
      : `<div class="panel">
           <h3>이어진 코드 ${explore.neighbors.length}건</h3>
           <label class="row" style="gap:8px;margin-bottom:12px">
             <span class="muted">깊이</span>
             <select id="depth">
               <option value="1"${explore.depth === 1 ? " selected" : ""}>1홉</option>
               <option value="2"${explore.depth === 2 ? " selected" : ""}>2홉</option>
               <option value="3"${explore.depth === 3 ? " selected" : ""}>3홉</option>
             </select>
           </label>
           ${
             explore.neighbors.length === 0
               ? `<p class="muted">호출·주입·API 호출로 이어진 것이 없습니다.</p>`
               : `<ul class="hits">${explore.neighbors.map((h) => hitRow(h, false, false)).join("")}</ul>`
           }
         </div>`;

  return `
    <h2>탐색</h2>
    <p class="lead">심볼을 찾고, 고른 심볼과 이어진 코드를 봅니다.</p>
    ${explore.error ? `<div class="error">${esc(explore.error)}</div>` : ""}

    <div class="panel">
      <h3>검색</h3>
      <div class="row">
        <input type="text" id="q" value="${esc(explore.query)}"
               placeholder="예: createOrder, 주문 취소, OrderController" />
        <button class="primary" id="do-search" ${explore.busy ? "disabled" : ""}>
          ${explore.busy ? "찾는 중입니다" : "찾기"}
        </button>
      </div>
      <p class="note">항목을 누르면 그 심볼을 호출하거나 주입받는 코드를 아래에 보여 줍니다.
        저장소가 여러 개면 저장소를 건너는 연결도 함께 나옵니다.
        깊이를 올리면 간접적으로 이어진 것까지 보이지만, 많이 쓰이는 함수에서는
        수백 건이 나옵니다.</p>
    </div>

    <div class="panel">
      <h3>결과${explore.hits ? ` ${explore.hits.length}건` : ""}</h3>
      ${results}
    </div>

    ${linked}`;
}

async function runSearch() {
  explore.busy = true;
  explore.error = null;
  explore.neighbors = null;
  explore.selected = null;
  show("explore");
  try {
    explore.hits = await invoke("search", { query: explore.query, limit: 100 });
  } catch (e) {
    explore.error = String(e);
    explore.hits = [];
  }
  explore.busy = false;
  show("explore");
}

async function pickHit(id) {
  explore.selected = id;
  explore.error = null;
  try {
    explore.neighbors = await invoke("neighbors", { id, depth: explore.depth });
  } catch (e) {
    explore.error = String(e);
    explore.neighbors = [];
  }
  show("explore");
}

function bindExplore() {
  const input = document.getElementById("q");
  if (input) {
    input.addEventListener("input", (e) => (explore.query = e.target.value));
    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter") runSearch();
    });
  }
  document.getElementById("do-search")?.addEventListener("click", runSearch);
  document.getElementById("depth")?.addEventListener("change", (e) => {
    explore.depth = Number(e.target.value);
    if (explore.selected) pickHit(explore.selected);
  });
  document.querySelectorAll("[data-pick]").forEach((li) =>
    li.addEventListener("click", () => pickHit(li.dataset.pick))
  );
}

// ── 팩 ───────────────────────────────────────────────────
/** 랭킹 가중치 다섯 개. 이름과 무엇을 보는 신호인지 함께 적는다. */
const WEIGHTS = [
  ["alpha_bm25", "α 어휘 일치", "질의에 적힌 단어가 이름과 얼마나 겹치는가"],
  ["beta_ppr", "β 그래프 근접", "시드에서 몇 걸음 안에 닿는가"],
  ["gamma_recency", "γ 최근성", "최근에 고친 코드인가"],
  ["delta_cochange", "δ 동시 변경", "시드와 늘 함께 바뀌어 왔는가"],
  ["epsilon_central", "ε 중심성", "많은 곳에서 쓰이는 코드인가"],
];

/** 설정을 읽기 전까지 쓰는 값. `RankWeights::default()`와 같다. */
const DEFAULT_WEIGHTS = {
  alpha_bm25: 0.7,
  beta_ppr: 0.5,
  gamma_recency: 0.3,
  delta_cochange: 0.4,
  epsilon_central: 0.2,
};

const packState = {
  task: "",
  budget: 4000,
  weights: { ...DEFAULT_WEIGHTS },
  /// 설정에 저장된 값을 이미 가져왔는가.
  loaded: false,
  view: null,
  busy: false,
  error: null,
  saved: null,
};

function packView() {
  if (!data || !data.config) return needSolution("팩");
  if (!data.index) return needIndex("팩");

  const w = packState.weights;

  const sliders = WEIGHTS.map(
    ([key, label, note]) => `
    <div class="slider">
      <div class="slider-head">
        <span>${label}</span>
        <span class="val" id="w-${key}">${w[key].toFixed(2)}</span>
      </div>
      <input type="range" data-weight="${key}" min="0" max="2" step="0.05" value="${w[key]}" />
      <div class="note">${note}</div>
    </div>`
  ).join("");

  return `
    <h2>팩</h2>
    <p class="lead">태스크 한 문장으로 에이전트에게 넘길 좌표 묶음을 만듭니다.</p>
    ${packState.error ? `<div class="error">${esc(packState.error)}</div>` : ""}
    ${packState.saved ? `<div class="ok-box">${esc(packState.saved)}</div>` : ""}

    <div class="panel">
      <h3>태스크</h3>
      <div class="row">
        <input type="text" id="task" value="${esc(packState.task)}"
               placeholder="예: 주문 취소할 때 재고를 되돌리는 곳을 고치고 싶다" />
        <button class="primary" id="do-pack" ${packState.busy ? "disabled" : ""}>
          ${packState.busy ? "만드는 중입니다" : "팩 만들기"}
        </button>
      </div>
      <label class="row" style="gap:8px;margin-top:12px">
        <span class="muted">토큰 예산</span>
        <input type="number" id="budget" value="${packState.budget}" min="500" max="60000"
               step="500" style="width:120px" />
      </label>
      <p class="note">예산은 상한이지 목표가 아닙니다. 관련 코드가 적으면 다 채우지 않고 끝냅니다.</p>
    </div>

    <div class="panel">
      <h3>랭킹 가중치</h3>
      <div class="sliders">${sliders}</div>
      <div class="actions" style="margin-top:14px">
        <button class="action" id="save-weights">공용 설정에 저장</button>
      </div>
      <p class="note">움직여 보고 결과가 나아지면 저장하십시오.
        <code>nunchi.shared.toml</code>에 들어가므로 커밋하면 다른 장비와 에이전트도 같은 값을 씁니다.</p>
    </div>

    ${packResult()}`;
}

function packResult() {
  const v = packState.view;
  if (!v) return "";
  const p = v.pack;

  if (p.hint) {
    return `<div class="panel"><h3>결과</h3><pre class="hint">${esc(p.hint)}</pre></div>`;
  }

  const pct = p.budget > 0 ? Math.round((p.used / p.budget) * 100) : 0;
  const rows = p.items
    .map((i) => {
      const why = Object.entries(i.why ?? {})
        .sort((a, b) => b[1] - a[1])
        .slice(0, 2)
        .map(([k, n]) => `${k} ${n.toFixed(2)}`)
        .join(" · ");
      return `<tr>
        <td><span class="tier t${esc(i.tier)}">${esc(i.tier)}</span></td>
        <td>${esc(i.sym)}<div class="muted">${esc(why)}</div></td>
        <td><code>${esc(i.ref)}</code></td>
        <td class="num">${num(i.tokens)}</td>
      </tr>`;
    })
    .join("");

  const cross = p.related?.cross_repo?.length
    ? `<div class="panel">
         <h3>저장소를 건너는 연결</h3>
         <p class="note">grep으로는 원리적으로 나오지 않는 정보입니다.</p>
         <ul class="hits">${p.related.cross_repo
           .map(
             (c) => `<li class="hit">
               <div class="hit-head"><span class="kind">${esc(c.repo)}</span>
                 <strong>${esc(c.sym)}</strong><span class="muted">${esc(c.via)}</span></div>
               <div class="ref"><code>${esc(c.ref)}</code></div>
             </li>`
           )
           .join("")}</ul>
       </div>`
    : "";

  const stale = p.stale?.length
    ? `<div class="panel">
         <h3>인덱스가 낡은 항목</h3>
         <p class="note">파일이 인덱싱 뒤에 바뀌었습니다. 좌표가 어긋났을 수 있으니 직접 확인하십시오.</p>
         <ul class="hits">${p.stale.map((s) => `<li class="hit"><code>${esc(s)}</code></li>`).join("")}</ul>
       </div>`
    : "";

  return `
    <div class="panel">
      <h3>결과</h3>
      <div class="bar"><span style="width:${Math.min(pct, 100)}%"></span></div>
      <p class="note">토큰 ${num(p.used)} / ${num(p.budget)} (${pct}%) · 항목 ${p.items.length}개</p>
      <div class="tags" style="margin:10px 0">
        ${p.seeds.map((s) => `<span class="tag">${esc(s)}</span>`).join("")}
      </div>
      <table>
        <thead><tr><th>단계</th><th>심볼</th><th>좌표</th><th class="num">토큰</th></tr></thead>
        <tbody>${rows}</tbody>
      </table>
    </div>
    ${cross}
    ${stale}
    <div class="panel">
      <h3>에이전트에게 넘어가는 형태</h3>
      <textarea class="raw" readonly rows="14">${esc(v.text)}</textarea>
      <div class="actions"><button class="action" id="copy-pack">복사</button></div>
    </div>`;
}

async function runPack() {
  packState.busy = true;
  packState.error = null;
  packState.saved = null;
  show("pack");
  try {
    packState.view = await invoke("build_pack", {
      task: packState.task,
      budget: packState.budget,
      weights: packState.weights,
    });
  } catch (e) {
    packState.error = String(e);
    packState.view = null;
  }
  packState.busy = false;
  show("pack");
}

function bindPack() {
  const task = document.getElementById("task");
  if (task) {
    task.addEventListener("input", (e) => (packState.task = e.target.value));
    task.addEventListener("keydown", (e) => {
      if (e.key === "Enter") runPack();
    });
  }
  document.getElementById("budget")?.addEventListener("input", (e) => {
    packState.budget = Number(e.target.value) || 4000;
  });
  document.getElementById("do-pack")?.addEventListener("click", runPack);

  // 슬라이더는 화면을 다시 그리지 않고 숫자만 바꾼다. 다시 그리면 손을 뗀 것처럼 된다.
  document.querySelectorAll("[data-weight]").forEach((r) =>
    r.addEventListener("input", (e) => {
      const key = r.dataset.weight;
      packState.weights[key] = Number(e.target.value);
      document.getElementById(`w-${key}`).textContent = Number(e.target.value).toFixed(2);
    })
  );

  document.getElementById("save-weights")?.addEventListener("click", async () => {
    packState.error = null;
    try {
      const path = await invoke("save_weights", { weights: packState.weights });
      packState.saved = `${path} 에 저장했습니다.`;
    } catch (e) {
      packState.error = String(e);
    }
    show("pack");
  });

  document.getElementById("copy-pack")?.addEventListener("click", () => {
    const box = document.querySelector("textarea.raw");
    if (!box) return;
    box.select();
    navigator.clipboard?.writeText(box.value).catch(() => {});
  });
}

/** 팩 화면을 처음 열 때 설정에 저장된 가중치를 가져온다. */
async function loadWeights() {
  if (packState.loaded) return;
  try {
    packState.weights = await invoke("pack_defaults");
    packState.loaded = true;
    show("pack");
  } catch {
    // 아직 인덱싱하지 않으면 실패한다. 그 화면은 어차피 안내만 보여 준다.
  }
}

// ── 설정 ─────────────────────────────────────────────────
const settingsState = {
  tab: "form",
  form: null,
  raw: null,
  which: "shared",
  error: null,
  saved: null,
};

function settingsView() {
  if (!data || !data.config) return needSolution("설정");

  const tabs = `
    <div class="tabs">
      <button class="tab${settingsState.tab === "form" ? " on" : ""}" data-tab="form">기본</button>
      <button class="tab${settingsState.tab === "shared" ? " on" : ""}" data-tab="shared">nunchi.shared.toml</button>
      <button class="tab${settingsState.tab === "local" ? " on" : ""}" data-tab="local">nunchi.toml</button>
    </div>`;

  const body = settingsState.tab === "form" ? formTab() : rawTab();

  return `
    <h2>설정</h2>
    <p class="lead">자주 고치는 값은 폼으로, 프레임워크 규칙처럼 구조가 깊은 것은 원문으로 고칩니다.</p>
    ${settingsState.error ? `<div class="error">${esc(settingsState.error)}</div>` : ""}
    ${settingsState.saved ? `<div class="ok-box">${esc(settingsState.saved)}</div>` : ""}
    ${tabs}
    ${body}`;
}

function formTab() {
  const f = settingsState.form;
  if (!f) return `<p class="muted">불러오는 중입니다.</p>`;

  const syn = f.synonyms.length
    ? f.synonyms
        .map(
          (s, i) => `<div class="row syn" style="margin-bottom:6px">
            <input type="text" data-syn-term="${i}" value="${esc(s.term)}"
                   placeholder="댓글" style="max-width:160px" />
            <input type="text" data-syn-words="${i}" value="${esc(s.words.join(", "))}"
                   placeholder="comment, reply" />
            <button class="remove" data-syn-del="${i}">제거</button>
          </div>`
        )
        .join("")
    : `<p class="muted">아직 등록한 용어가 없습니다.</p>`;

  return `
    <div class="panel">
      <h3>솔루션</h3>
      <label class="field"><span>이름</span>
        <input type="text" id="f-name" value="${esc(f.name)}" /></label>
      <p class="note">저장소 경로는 개요 화면의 저장소 변경에서 고칩니다.</p>
    </div>

    <div class="panel">
      <h3>인덱싱</h3>
      <label class="field"><span>언어 (쉼표로 구분)</span>
        <input type="text" id="f-langs" value="${esc(f.languages.join(", "))}" /></label>
      <label class="field"><span>제외 패턴 (한 줄에 하나)</span>
        <textarea id="f-exclude" rows="8" class="raw">${esc(f.exclude.join("\n"))}</textarea></label>
      <p class="note">생성 코드와 벤더 디렉터리가 들어오면 랭킹이 오염됩니다.</p>
      <div class="grid3">
        <label class="field"><span>파일 크기 상한 (MB)</span>
          <input type="number" id="f-bytes" min="0.1" step="0.1"
                 value="${(f.max_file_bytes / (1024 * 1024)).toFixed(1)}" /></label>
        <label class="field"><span>읽을 커밋 수</span>
          <input type="number" id="f-commits" min="0" step="100" value="${f.max_commits}" /></label>
        <label class="field"><span>이름이 겹칠 때 후보 상한</span>
          <input type="number" id="f-candidates" min="1" step="1" value="${f.max_candidates}" /></label>
      </div>
      <p class="note">tree-sitter는 타입을 모르므로 이름이 같은 후보를 전부 내놓습니다.
        후보 상한을 올리면 애매한 엣지가 늘어나는 대신 놓치는 호출이 줄어듭니다.
        애매한 엣지는 신뢰도가 후보 수만큼 나뉘어 붙으므로 랭킹에서 확실한 엣지에 밀립니다.</p>
    </div>

    <div class="panel">
      <h3>도메인 용어 사전</h3>
      ${syn}
      <button class="action" id="syn-add" style="margin-top:8px">용어 추가</button>
      <p class="note">인덱스는 영어 식별자로 되어 있습니다. 한국어로 찾으려면
        여기에 등록해야 검색과 팩이 함께 넓어집니다.</p>
    </div>

    <div class="panel">
      <h3>랭킹 가중치</h3>
      <dl>
        ${WEIGHTS.map(([k, label]) => `<dt>${label}</dt><dd>${f.rank[k].toFixed(2)}</dd>`).join("")}
      </dl>
      <p class="note">가중치는 팩 화면에서 결과를 보며 맞추는 편이 낫습니다. 여기서는 값만 보여 줍니다.</p>
    </div>

    <div class="actions">
      <button class="primary" id="f-save">저장</button>
      <button class="action" id="f-reload">되돌리기</button>
    </div>
    <p class="note">두 파일에 나누어 씁니다. 경로가 없는 값은 공유하는
      <code>nunchi.shared.toml</code>에도 넣습니다. 불러올 때 공용 파일이 나중에 덮어쓰기 때문입니다.</p>`;
}

function rawTab() {
  const r = settingsState.raw;
  if (!r) return `<p class="muted">불러오는 중입니다.</p>`;
  return `
    <div class="panel">
      <h3><code>${esc(r.path)}</code></h3>
      ${r.exists ? "" : `<p class="note">아직 없는 파일입니다. 저장하면 만들어집니다.</p>`}
      <textarea id="raw-text" class="raw" rows="26" spellcheck="false">${esc(r.text)}</textarea>
      <div class="actions">
        <button class="primary" id="raw-save">저장</button>
        <button class="action" id="raw-reload">되돌리기</button>
      </div>
      <p class="note">저장하기 전에 설정 형식으로 읽어 봅니다. 형식이 맞지 않으면 파일을 건드리지 않습니다.
        ${
          settingsState.which === "shared"
            ? "프레임워크 규칙을 여기에 추가하면 다시 빌드하지 않아도 지원 범위가 넓어집니다."
            : "저장소 경로가 들어 있어 커밋하지 않는 파일입니다."
        }</p>
    </div>`;
}

/** 화면의 입력값을 상태로 옮긴다. 줄을 더하거나 뺄 때 지금 적은 것을 잃지 않게 한다. */
function collectForm() {
  const f = settingsState.form;
  if (!f || settingsState.tab !== "form") return;
  const get = (id) => document.getElementById(id)?.value;
  f.name = get("f-name") ?? f.name;
  f.languages = (get("f-langs") ?? "")
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
  f.exclude = (get("f-exclude") ?? "")
    .split("\n")
    .map((s) => s.trim())
    .filter(Boolean);
  const mb = parseFloat(get("f-bytes"));
  if (!Number.isNaN(mb)) f.max_file_bytes = Math.round(mb * 1024 * 1024);
  f.max_commits = Number(get("f-commits")) || 0;
  f.max_candidates = Number(get("f-candidates")) || 1;
  f.synonyms = f.synonyms.map((s, i) => ({
    term: document.querySelector(`[data-syn-term="${i}"]`)?.value ?? s.term,
    words: (document.querySelector(`[data-syn-words="${i}"]`)?.value ?? "")
      .split(",")
      .map((w) => w.trim())
      .filter(Boolean),
  }));
}

async function loadSettings() {
  settingsState.error = null;
  try {
    if (settingsState.tab === "form") {
      settingsState.form = await invoke("settings_read");
    } else {
      settingsState.raw = await invoke("read_toml", { which: settingsState.which });
    }
  } catch (e) {
    settingsState.error = String(e);
  }
  show("settings");
}

function bindSettings() {
  document.querySelectorAll("[data-tab]").forEach((b) =>
    b.addEventListener("click", () => {
      collectForm();
      settingsState.saved = null;
      settingsState.error = null;
      settingsState.tab = b.dataset.tab;
      if (settingsState.tab !== "form") {
        settingsState.which = settingsState.tab;
        settingsState.raw = null;
      }
      show("settings");
      loadSettings();
    })
  );

  document.getElementById("syn-add")?.addEventListener("click", () => {
    collectForm();
    settingsState.form.synonyms.push({ term: "", words: [] });
    show("settings");
  });
  document.querySelectorAll("[data-syn-del]").forEach((b) =>
    b.addEventListener("click", () => {
      collectForm();
      settingsState.form.synonyms.splice(Number(b.dataset.synDel), 1);
      show("settings");
    })
  );

  document.getElementById("f-save")?.addEventListener("click", async () => {
    collectForm();
    settingsState.error = null;
    settingsState.saved = null;
    try {
      data = await invoke("settings_save", { form: settingsState.form });
      settingsState.saved = "저장했습니다. 다음 인덱싱부터 반영됩니다.";
      setSolutionLabel();
    } catch (e) {
      settingsState.error = String(e);
    }
    show("settings");
  });
  document.getElementById("f-reload")?.addEventListener("click", () => {
    settingsState.saved = null;
    loadSettings();
  });

  document.getElementById("raw-save")?.addEventListener("click", async () => {
    settingsState.error = null;
    settingsState.saved = null;
    const text = document.getElementById("raw-text")?.value ?? "";
    settingsState.raw.text = text;
    try {
      data = await invoke("save_toml", { which: settingsState.which, text });
      settingsState.saved = "저장했습니다.";
      setSolutionLabel();
    } catch (e) {
      settingsState.error = String(e);
    }
    show("settings");
  });
  document.getElementById("raw-reload")?.addEventListener("click", () => {
    settingsState.saved = null;
    settingsState.raw = null;
    show("settings");
    loadSettings();
  });
  document.getElementById("raw-text")?.addEventListener("input", (e) => {
    settingsState.raw.text = e.target.value;
  });
}

const views = {
  overview: overviewView,
  solutions: solutionsView,
  setup: setupView,
  index: indexView,
  explore: exploreView,
  pack: packView,
  settings: settingsView,
};

/** 기존 설정을 마법사에 채워 넣고 그 화면으로 옮긴다. */
function editRepos() {
  const c = data.config;
  draft.repos = c.repos.map((r) => r.path);
  draft.name = c.solution;
  // 설정 파일이 있던 자리에 다시 만든다.
  draft.dir = c.path.replace(/[/\\]nunchi\.toml$/, "");
  draft.languages = null;
  draft.error = null;
  draft.overwrite = true;
  refreshLanguages().then(() => show("setup"));
}

function show(name) {
  // 다시 그리면 입력 칸의 포커스와 커서 위치가 사라진다. 글자를 넣는 도중에
  // 결과가 도착하면 타이핑이 끊기므로 되돌려 놓는다.
  const focused = document.activeElement?.id;
  const caret = document.activeElement?.selectionStart;

  document.querySelectorAll(".nav").forEach((b) => {
    if (b.dataset.view === name) b.setAttribute("aria-current", "page");
    else b.removeAttribute("aria-current");
  });
  document.getElementById("view").innerHTML = views[name]();
  if (name === "setup") bindSetup();
  if (name === "solutions") bindSolutions();
  if (name === "index") bindIndex();
  if (name === "explore") bindExplore();
  if (name === "pack") bindPack();
  if (name === "settings") bindSettings();
  document.getElementById("edit-repos")?.addEventListener("click", editRepos);
  document.getElementById("go-index")?.addEventListener("click", () => enter("index"));

  if (focused) {
    const again = document.getElementById(focused);
    if (again) {
      again.focus();
      if (caret != null && again.setSelectionRange) {
        try {
          again.setSelectionRange(caret, caret);
        } catch {
          // 범위를 다룰 수 없는 입력 종류가 있다. 포커스만 살리면 충분하다.
        }
      }
    }
  }
}

/** 화면을 처음 열 때 필요한 것을 가져온다. */
function enter(name) {
  show(name);
  if (name === "pack") loadWeights();
  if (name === "settings" && !settingsState.form && !settingsState.raw) loadSettings();
}

/** 화면을 띄우지 못한 이유를 그대로 보여 준다. 흰 화면으로 두면 원인을 알 수 없다. */
function fatal(message) {
  document.getElementById("view").innerHTML = `
    <h2>화면을 띄우지 못했습니다</h2>
    <div class="error">${esc(message)}</div>`;
}

async function start() {
  if (!invoke) {
    fatal("Tauri API 를 찾지 못했습니다. tauri.conf.json 의 withGlobalTauri 설정을 확인하십시오.");
    return;
  }
  document.querySelectorAll(".nav").forEach((b) =>
    b.addEventListener("click", () => enter(b.dataset.view))
  );
  document.getElementById("solution-label").addEventListener("click", () => show("solutions"));
  await listenIndexEvents();

  let boot;
  try {
    boot = await invoke("startup");
  } catch (e) {
    fatal(String(e));
    return;
  }
  recent = boot.recent;
  data = boot.opened;
  // 마지막에 열었던 솔루션이 있으면 바로 개요를 보여 주고,
  // 없으면 무엇을 열지 고르는 화면으로 간다.
  if (data) {
    setSolutionLabel();
    show("overview");
  } else {
    show("solutions");
  }
}

start();

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

    ${index}`;
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
  if (!data || !data.config) {
    return `<h2>인덱싱</h2>
      <div class="empty">
        <strong>먼저 솔루션을 여십시오.</strong>
        왼쪽 아래의 솔루션 이름을 누르면 목록이 나옵니다.
      </div>`;
  }

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

// ── 아직 만들지 않은 화면 ────────────────────────────────
const soon = (title, note) => {
  if (!data || !data.config) {
    return `<h2>${title}</h2>
      <div class="empty">
        <strong>먼저 솔루션을 여십시오.</strong>
        왼쪽 아래의 솔루션 이름을 누르면 목록이 나옵니다.
      </div>`;
  }
  return `<h2>${title}</h2><div class="soon">${note}</div>`;
};

const views = {
  overview: overviewView,
  solutions: solutionsView,
  setup: setupView,
  index: indexView,
  explore: () => soon("탐색", "심볼과 이웃을 찾는 화면입니다."),
  pack: () => soon("팩", "질의를 넣고 가중치를 조정하며 결과를 봅니다."),
  settings: () => soon("설정", "설정 파일을 폼으로 편집합니다."),
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
  document.querySelectorAll(".nav").forEach((b) => {
    if (b.dataset.view === name) b.setAttribute("aria-current", "page");
    else b.removeAttribute("aria-current");
  });
  document.getElementById("view").innerHTML = views[name]();
  if (name === "setup") bindSetup();
  if (name === "solutions") bindSolutions();
  if (name === "index") bindIndex();
  document.getElementById("edit-repos")?.addEventListener("click", editRepos);
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
    b.addEventListener("click", () => show(b.dataset.view))
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

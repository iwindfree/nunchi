const { invoke } = window.__TAURI__.core;

/** 화면 사이에서 공유하는 상태. 인덱싱이 끝나면 다시 불러 갱신한다. */
let data = null;

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
  if (!data.config) {
    // 설정이 없으면 개요 대신 마법사를 보여 준다.
    return setupView();
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


// ── 초기 설정 마법사 ─────────────────────────────────────
/** 설정을 만들기 전까지 들고 있는 입력값. */
const draft = { dir: "", repos: [], name: "", languages: null, error: null, busy: false };

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
    const out = await invoke("init_solution", {
      dir: draft.dir,
      repos: draft.repos,
      name: draft.name,
      force: false,
    });
    // 설정이 생겼으니 개요를 다시 읽어 화면을 갱신한다.
    data = await invoke("overview");
    document.getElementById("solution-label").textContent = out.solution;
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
    <h2>시작하기</h2>
    <p class="lead">인덱싱할 저장소를 고르면 설정 파일을 만들어 드립니다.</p>

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
          <input type="text" id="dir" value="${esc(draft.dir)}" readonly />
          <button class="action" id="pick-dir">변경</button>
        </div>
      </label>
      <p class="note">이 위치에 <code>nunchi.toml</code>과 <code>nunchi.shared.toml</code>이 생깁니다.
        앞의 것은 경로가 들어 있어 커밋하지 않고, 뒤의 것은 공유합니다.</p>
    </div>

    <div class="actions">
      <button class="primary" id="create" ${ready ? "" : "disabled"}>
        ${draft.busy ? "만드는 중입니다" : "설정 만들기"}
      </button>
    </div>`;
}

/** 마법사 화면의 버튼과 입력을 연결한다. */
function bindSetup() {
  document.getElementById("add-repo")?.addEventListener("click", pickRepo);
  document.getElementById("pick-dir")?.addEventListener("click", pickDir);
  document.getElementById("create")?.addEventListener("click", createConfig);
  document.querySelectorAll("[data-remove]").forEach((b) =>
    b.addEventListener("click", () => removeRepo(b.dataset.remove))
  );
  const nameInput = document.getElementById("sol-name");
  if (nameInput) nameInput.addEventListener("input", (e) => (draft.name = e.target.value));
}

// ── 아직 만들지 않은 화면 ────────────────────────────────
const soon = (title, note) => `
  <h2>${title}</h2>
  <div class="soon">${note}</div>`;

const views = {
  overview: overviewView,
  setup: setupView,
  index: () => soon("인덱싱", "다음 단계에서 실행 버튼과 진행 표시를 붙입니다."),
  explore: () => soon("탐색", "심볼과 이웃을 찾는 화면입니다."),
  pack: () => soon("팩", "질의를 넣고 가중치를 조정하며 결과를 봅니다."),
  settings: () => soon("설정", "설정 파일을 폼으로 편집합니다."),
};

function show(name) {
  document.querySelectorAll(".nav").forEach((b) => {
    if (b.dataset.view === name) b.setAttribute("aria-current", "page");
    else b.removeAttribute("aria-current");
  });
  document.getElementById("view").innerHTML = views[name]();
  if (name === "setup" || (name === "overview" && !data.config)) bindSetup();
}

async function start() {
  data = await invoke("overview");
  draft.dir = await invoke("default_dir");
  const label = document.getElementById("solution-label");
  label.textContent = data.config ? data.config.solution : "설정 없음";
  document.querySelectorAll(".nav").forEach((b) =>
    b.addEventListener("click", () => show(b.dataset.view))
  );
  show("overview");
}

start();

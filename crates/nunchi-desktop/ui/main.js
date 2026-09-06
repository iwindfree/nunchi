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
  if (data.problem) {
    return `
      <h2>개요</h2>
      <div class="empty">
        <strong>시작할 준비가 되지 않았습니다.</strong>
        ${esc(data.problem)}
      </div>`;
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

// ── 아직 만들지 않은 화면 ────────────────────────────────
const soon = (title, note) => `
  <h2>${title}</h2>
  <div class="soon">${note}</div>`;

const views = {
  overview: overviewView,
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
}

async function start() {
  data = await invoke("overview");
  const label = document.getElementById("solution-label");
  label.textContent = data.config ? data.config.solution : "설정 없음";
  document.querySelectorAll(".nav").forEach((b) =>
    b.addEventListener("click", () => show(b.dataset.view))
  );
  show("overview");
}

start();

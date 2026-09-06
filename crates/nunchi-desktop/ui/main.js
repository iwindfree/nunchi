const { invoke } = window.__TAURI__.core;

/** 화면을 처음 그린다. 지금은 상태 확인만 한다. */
async function render() {
  const app = document.getElementById("app");
  let s;
  try {
    s = await invoke("status");
  } catch (e) {
    app.innerHTML = `<div class="panel"><h2>오류</h2><p>${e}</p></div>`;
    return;
  }

  const repos = s.repos.length
    ? s.repos.map((r) => `<div><code>${r}</code></div>`).join("")
    : "<span>등록된 저장소가 없습니다.</span>";

  const solution = s.config_found
    ? `<div class="panel">
         <h2>솔루션</h2>
         <dl>
           <dt>이름</dt><dd>${s.solution ?? ""}</dd>
           <dt>설정 파일</dt><dd><code>${s.config_path ?? ""}</code></dd>
           <dt>저장소</dt><dd>${repos}</dd>
         </dl>
       </div>`
    : `<div class="empty">
         <strong>설정 파일을 찾지 못했습니다.</strong>
         저장소를 등록하면 인덱싱을 시작할 수 있습니다.
       </div>`;

  app.innerHTML = `
    ${solution}
    <div class="panel">
      <h2>적용 중인 규칙</h2>
      <dl>
        <dt>프레임워크 규칙</dt><dd>${s.rule_count}개</dd>
        <dt>지원 언어</dt><dd>${s.language_count}개</dd>
      </dl>
    </div>`;
}

render();

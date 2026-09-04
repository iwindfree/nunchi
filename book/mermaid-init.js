// mermaid 다이어그램을 그립니다.
//
// mdbook-mermaid 전처리기 없이 동작합니다. 전처리기는 원고의 ```mermaid 를
// <pre class="mermaid"> 로 바꿔 주는 일만 하는데, 그 변환을 브라우저에서
// 직접 하면 됩니다.
//
// 이렇게 하면 두 가지를 동시에 얻습니다.
//   1. 빌드에 별도 실행 파일이 필요 없습니다
//   2. 원고에 ```mermaid 를 그대로 두므로 GitHub 에서도 그림이 보입니다
//      (GitHub 은 mermaid 블록을 자체적으로 그립니다)

(function () {
  var DARK_THEMES = ["ayu", "coal", "navy"];

  // 원본 소스를 보관합니다. 테마가 바뀌면 다시 그려야 하는데,
  // 이미 SVG 로 바뀐 뒤에는 원본을 되찾을 수 없기 때문입니다.
  var sources = [];

  function collect() {
    // mdBook 은 ```mermaid 를 <pre><code class="language-mermaid"> 로 만듭니다.
    var blocks = document.querySelectorAll("pre > code.language-mermaid");
    for (var i = 0; i < blocks.length; i++) {
      var code = blocks[i];
      var pre = code.parentElement;
      var holder = document.createElement("pre");
      holder.className = "mermaid";
      // textContent 를 쓰면 &lt; 같은 실체 참조가 원래 문자로 돌아옵니다.
      holder.textContent = code.textContent;
      pre.parentElement.replaceChild(holder, pre);
    }
    sources = [];
    var holders = document.querySelectorAll("pre.mermaid");
    for (var j = 0; j < holders.length; j++) {
      sources.push({ el: holders[j], text: holders[j].textContent });
    }
  }

  function isDark() {
    var cls = document.documentElement.classList;
    for (var i = 0; i < DARK_THEMES.length; i++) {
      if (cls.contains(DARK_THEMES[i])) {
        return true;
      }
    }
    return false;
  }

  function render() {
    if (typeof mermaid === "undefined") {
      return;
    }
    mermaid.initialize({
      startOnLoad: false,
      theme: isDark() ? "dark" : "default",
    });
    // 보관해 둔 원본으로 되돌린 뒤 다시 그립니다.
    var nodes = [];
    for (var i = 0; i < sources.length; i++) {
      var s = sources[i];
      s.el.removeAttribute("data-processed");
      s.el.textContent = s.text;
      nodes.push(s.el);
    }
    if (nodes.length) {
      mermaid.init(undefined, nodes);
    }
  }

  function watchTheme() {
    // mdBook 의 테마 선택기는 페이지를 새로 고치지 않고 <html> 의 클래스만
    // 바꿉니다. 그대로 두면 다이어그램만 옛 테마로 남아 읽기 어려워집니다.
    var last = isDark();
    new MutationObserver(function () {
      var now = isDark();
      if (now !== last) {
        last = now;
        render();
      }
    }).observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    });
  }

  function start() {
    collect();
    render();
    watchTheme();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start);
  } else {
    start();
  }
})();

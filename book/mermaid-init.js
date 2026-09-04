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
  function convertFences() {
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
  }

  function start() {
    convertFences();
    var dark = document.documentElement.classList.contains("ayu")
      || document.documentElement.classList.contains("coal")
      || document.documentElement.classList.contains("navy");
    mermaid.initialize({
      startOnLoad: true,
      theme: dark ? "dark" : "default",
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start);
  } else {
    start();
  }
})();

//! 언어 판별. v1 대상은 Java · TypeScript · Rust (docs/CONTRIBUTING.md 개발 환경 도그푸딩 포함).

use std::path::Path;

/// 확장자 → 언어 이름. 인덱싱 대상이 아니면 `None`.
pub fn detect(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    Some(match ext.as_str() {
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "ts" | "tsx" | "mts" | "cts" => "typescript",
        "js" | "jsx" | "mjs" | "cjs" => "javascript",
        "rs" => "rust",
        "py" | "pyi" => "python",
        "cs" => "csharp",
        "sql" => "sql",
        "md" | "mdx" => "markdown",
        "yml" | "yaml" => "yaml",
        "toml" => "toml",
        "json" => "json",
        "xml" => "xml",
        "gradle" => "gradle",
        "properties" => "properties",
        _ => return None,
    })
}

/// `nunchi doctor`가 커버리지를 계산할 때 "파서가 있어야 하는" 언어인지 판단한다.
/// 설정 파일류는 파싱 실패해도 문제가 아니다.
pub fn is_code(lang: &str) -> bool {
    matches!(
        lang,
        "java" | "kotlin" | "typescript" | "javascript" | "rust" | "csharp" | "python"
    )
}

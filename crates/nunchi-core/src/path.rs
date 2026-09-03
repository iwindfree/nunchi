//! 경로 정규화 — Windows 대응 (docs/CONTRIBUTING.md 개발 환경)
//!
//! 회사 컴이 Windows이므로 다음을 흡수해야 한다:
//! - 역슬래시 구분자 → 노드 ID가 플랫폼마다 달라지면 안 된다
//! - NTFS 대소문자 비구분(보존형) → 비교는 소문자, 표시는 원본
//! - MAX_PATH 260 제한 → Spring 깊은 패키지 + Gradle `build/`에서 실제로 걸린다

use std::path::{Path, PathBuf};

/// 표시·저장용 정규화. 항상 슬래시 구분자, 원본 대소문자 보존.
pub fn normalize(path: &Path) -> String {
    let s = path.to_string_lossy();
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    s.replace('\\', "/")
}

/// 저장소 루트 기준 상대 경로로 정규화. 루트 밖이면 `None`.
pub fn relative_to(root: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    Some(normalize(rel))
}

/// 비교·조회 전용 키. Windows 대소문자 비구분 파일시스템에서
/// 같은 파일이 서로 다른 노드로 갈라지는 것을 막는다.
///
/// 표시용 경로를 이 값으로 대체하면 안 된다 — 원본 대소문자를 잃는다.
pub fn compare_key(normalized: &str) -> String {
    normalized.to_lowercase()
}

/// Windows에서 260자를 넘는 절대 경로에 `\\?\` 확장 접두를 붙인다.
/// 다른 플랫폼에서는 그대로 돌려준다.
#[cfg(windows)]
pub fn to_extended_length(path: &Path) -> PathBuf {
    const MAX_PATH_MARGIN: usize = 240;
    let s = path.to_string_lossy();
    if s.len() < MAX_PATH_MARGIN || s.starts_with(r"\\?\") || !path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(format!(r"\\?\{}", s.replace('/', "\\")))
}

#[cfg(not(windows))]
pub fn to_extended_length(path: &Path) -> PathBuf {
    path.to_path_buf()
}

/// 워킹트리 파일 내용 해시.
///
/// **git blob SHA를 쓰지 않는다.** `core.autocrlf=true`인 Windows에서는
/// 워킹트리가 CRLF, blob이 LF라 두 값이 갈린다. 우리가 실제로 파싱하는 것은
/// 워킹트리 내용이므로 그것을 해시해 자기 일관성을 유지한다(docs/CONTRIBUTING.md 개발 환경).
pub fn content_hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_backslashes() {
        assert_eq!(normalize(Path::new(r"src\main\java\App.java")), "src/main/java/App.java");
    }

    #[test]
    fn strips_extended_length_prefix() {
        assert_eq!(normalize(Path::new(r"\\?\C:\repo\a.rs")), "C:/repo/a.rs");
    }

    #[test]
    fn compare_key_is_case_insensitive() {
        assert_eq!(compare_key("Src/App.java"), compare_key("src/app.java"));
    }

    #[test]
    fn relative_paths_are_repo_rooted() {
        let root = Path::new("/repo");
        assert_eq!(
            relative_to(root, Path::new("/repo/src/App.java")).as_deref(),
            Some("src/App.java")
        );
        assert_eq!(relative_to(root, Path::new("/other/App.java")), None);
    }

    #[test]
    fn content_hash_is_stable() {
        assert_eq!(content_hash(b"hello"), content_hash(b"hello"));
        assert_ne!(content_hash(b"hello"), content_hash(b"hello\r\n"));
    }
}

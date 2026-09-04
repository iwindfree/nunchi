// 아래 함수는 컴파일되지 않습니다.
//
// 목록을 순회하면서 조건에 맞는 것을 지우려고 합니다.
// 읽는 중에 바꾸려고 하므로 빌림 규칙에 걸립니다(1.3장).
//
// Vec 에 있는 메서드 하나로 한 줄에 해결할 수 있습니다.
// 힌트: "retain" 을 찾아보십시오.

pub fn drop_tests(paths: &mut Vec<String>) {
    for (i, p) in paths.iter().enumerate() {
        if p.contains("/test/") {
            paths.remove(i); // TODO: 이 줄에서 오류가 납니다
        }
    }
}

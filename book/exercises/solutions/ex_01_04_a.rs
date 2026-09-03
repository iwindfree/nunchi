// 정답
//
// 세 clone 을 없앤 이유는 각각 다릅니다.
//
// 1. repo.clone()
//    format! 은 값을 빌려서 읽기만 합니다. 소유권이 필요 없습니다.
//
// 2. files.clone()
//    count_files 가 개수만 세고 목록을 보관하지 않습니다.
//    그러므로 서명을 &[String] 으로 바꿔 빌려 받게 만듭니다.
//
// 3. first.clone()
//    2번과 같습니다. format! 은 읽기만 합니다.
//
// `files.first().cloned()` 에 남은 cloned 는 필요합니다.
// first() 가 Option<&String> 을 돌려주는데 우리는 String 이 필요하기
// 때문입니다. 이것은 정당한 복사입니다.

pub fn build_report(repo: String, files: Vec<String>) -> String {
    let header = format!("repo: {}", repo);
    let count = count_files(&files);
    let first = files.first().cloned().unwrap_or_default();
    format!("{header}\nfiles: {count}\nfirst: {}", first)
}

fn count_files(files: &[String]) -> usize {
    files.len()
}

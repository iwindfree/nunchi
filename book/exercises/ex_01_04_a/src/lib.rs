// 아래 코드는 컴파일됩니다. 다만 불필요한 clone 이 세 개 있습니다.
//
// 세 개를 모두 없애십시오. 테스트는 그대로 통과해야 합니다.
//
// 힌트: 어떤 함수가 값을 보관하고 어떤 함수가 읽기만 하는지 보십시오.
//       읽기만 하는 곳에는 clone 이 필요 없습니다(1.4장).

pub fn build_report(repo: String, files: Vec<String>) -> String {
    let header = format!("repo: {}", repo.clone());       // TODO
    let count = count_files(files.clone());               // TODO
    let first = files.first().cloned().unwrap_or_default();
    format!("{header}\nfiles: {count}\nfirst: {}", first.clone()) // TODO
}

fn count_files(files: Vec<String>) -> usize {
    files.len()
}

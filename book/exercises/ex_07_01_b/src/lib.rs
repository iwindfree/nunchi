// use 로 경로를 줄이는 문제입니다.
//
// 아래 코드는 컴파일되지만 같은 긴 경로를 계속 반복해서 씁니다.
// 파일 맨 위에 use 를 추가하고 본문에서 짧은 이름을 쓰도록 고치십시오.
//
// 결과가 같아야 하며 테스트는 그대로 통과해야 합니다.

// TODO: 여기에 use 를 추가하십시오

pub fn count_unique(names: &[&str]) -> usize {
    let mut set: std::collections::HashSet<String> = std::collections::HashSet::new();
    for n in names {
        set.insert(n.to_string());
    }
    set.len()
}

pub fn group(names: &[&str]) -> std::collections::HashMap<usize, Vec<String>> {
    let mut map: std::collections::HashMap<usize, Vec<String>> =
        std::collections::HashMap::new();
    for n in names {
        map.entry(n.len()).or_default().push(n.to_string());
    }
    map
}

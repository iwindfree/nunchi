// 정답
//
//     paths
//         .iter()
//         .filter_map(|p| p.rsplit_once('.').map(|(_, ext)| ext.to_string()))
//         .collect()
//
// filter_map 은 Option 을 돌려주는 함수를 받아, Some 인 것만 남기고
// 값을 꺼내 줍니다. map + filter + map 세 단계가 한 단계가 됩니다.
//
// unwrap() 이 사라진 점이 중요합니다. unwrap 은 값이 없으면 프로그램을
// 멈추므로 실제 코드에서 되도록 피합니다(2.1장). filter_map 을 쓰면
// 그럴 자리가 아예 생기지 않습니다.
//
// nunchi 의 pack.rs 에 있는 repo_roots 가 이 형태이며, 클로저 안에서
// ? 를 써서 더 짧게 만들었습니다.

pub fn extensions(paths: &[String]) -> Vec<String> {
    paths
        .iter()
        .filter_map(|p| p.rsplit_once('.').map(|(_, ext)| ext.to_string()))
        .collect()
}

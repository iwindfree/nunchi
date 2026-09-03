// 정답
//
// filter_map 안에서 ? 를 씁니다. 클로저도 함수이므로 Option 을 돌려주면
// ? 가 동작합니다(2.3장).
//
// rsplit('/').next() 는 마지막 조각을 줍니다. "/" 의 경우 빈 문자열이
// 나오므로 filter 로 걸러 냅니다.
//
// 마지막 collect() 가 HashMap 을 만듭니다. (키, 값) 짝을 모으면 표가
// 된다는 규칙 덕분에 따로 넣는 코드를 쓰지 않아도 됩니다.
//
// 반환 타입이 HashMap<String, String> 으로 적혀 있으므로 collect 가
// 무엇을 만들지 알 수 있습니다(4.3장).

use std::collections::HashMap;

pub fn repo_names(paths: &[String]) -> HashMap<String, String> {
    paths
        .iter()
        .filter_map(|p| {
            let name = p.rsplit('/').next()?;
            if name.is_empty() {
                return None;
            }
            Some((name.to_string(), p.clone()))
        })
        .collect()
}

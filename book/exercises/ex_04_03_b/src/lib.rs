// 아래 함수는 동작하지만 세 단계를 거칩니다.
//
// filter_map 으로 한 단계로 줄이십시오(4.3장).
//
// 하는 일: 경로 목록에서 확장자만 뽑습니다. 확장자가 없는 파일은 버립니다.

pub fn extensions(paths: &[String]) -> Vec<String> {
    paths
        .iter()
        .map(|p| p.rsplit_once('.').map(|(_, ext)| ext.to_string()))
        .filter(|o| o.is_some())
        .map(|o| o.unwrap())
        .collect()
}

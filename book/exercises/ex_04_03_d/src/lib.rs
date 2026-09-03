// 아래 함수는 컴파일되고 테스트도 일부 통과합니다. 그런데 조용히 틀립니다.
//
// 설정 파일에서 읽은 숫자들을 전부 변환하는데, 잘못된 값이 있으면
// 그것을 알려야 합니다. 지금 코드는 실패한 항목을 말없이 버립니다.
// 설정에 오타가 있어도 사용자가 알 수 없습니다.
//
// 하나라도 실패하면 전체가 실패하도록 고치십시오.
//
// 힌트: filter_map 은 실패를 버립니다. collect 를 Result<Vec<_>, _> 로
//       모으면 하나라도 Err 인 순간 멈추고 그 오류를 돌려줍니다(4.3장).

pub fn parse_all(raw: &[String]) -> Result<Vec<u32>, std::num::ParseIntError> {
    // TODO: 실패를 버리지 말고 위로 올리십시오
    let values: Vec<u32> = raw
        .iter()
        .filter_map(|s| s.trim().parse::<u32>().ok())
        .collect();
    Ok(values)
}

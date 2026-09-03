// 정답
//
//     raw.iter().map(|s| s.trim().parse::<u32>()).collect()
//
// filter_map 과 무엇이 다른가:
//
//   filter_map(|s| ... .ok())   실패한 항목을 말없이 버립니다
//   map(|s| ...).collect()      하나라도 실패하면 전체가 Err 가 됩니다
//
// .ok() 는 Result 를 Option 으로 바꾸면서 오류 정보를 버립니다(2.2장).
// 그것이 적절한 경우도 있습니다. 파일 수정 시각을 못 읽으면 최근성 점수를
// 0 으로 두면 되므로 nunchi 는 그 자리에서 .ok() 를 씁니다.
//
// 하지만 설정 값은 다릅니다. 사용자가 오타를 냈는데 조용히 무시하면
// 왜 동작이 이상한지 알 수 없습니다. 이럴 때는 실패를 올려야 합니다.
//
// collect 가 Result<Vec<u32>, _> 를 만드는 이유:
//   반환 타입에 그렇게 적혀 있으므로 컴파일러가 그쪽으로 모읍니다.
//   반환 타입이 없는 자리에서는 터보피시로 알려 줍니다.
//
//       .collect::<Result<Vec<_>, _>>()
//
// 이 방식은 하나라도 Err 를 만나면 그 자리에서 멈춥니다. 나머지 항목은
// 아예 변환하지 않습니다.

pub fn parse_all(raw: &[String]) -> Result<Vec<u32>, std::num::ParseIntError> {
    raw.iter().map(|s| s.trim().parse::<u32>()).collect()
}

// nunchi 의 IndexStats 를 단순하게 만든 것입니다.
//
// 아래 함수를 완성하십시오. 통계 구조체를 빌려서 값을 갱신해야 합니다.
//
// 규칙:
//   - stats 를 소유권으로 받지 마십시오. 호출한 쪽에서 계속 써야 합니다.
//   - 새 구조체를 만들어 돌려주지 마십시오.
//
// 힌트: 값을 바꾸는 빌림은 &mut 입니다(1.3장).

#[derive(Debug, Default, PartialEq)]
pub struct IndexStats {
    pub files_indexed: usize,
    pub symbols: usize,
}

// TODO: 아래 함수의 서명과 본문을 완성하십시오.
//       파일 하나와 그 안의 심볼 개수를 통계에 더합니다.
pub fn record_file(stats: ???, symbol_count: usize) {
    todo!()
}

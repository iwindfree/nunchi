// 정답
//
// &mut IndexStats 를 받아서 필드를 직접 바꿉니다.
//
// nunchi 의 실제 코드도 이 방식입니다. scan_repo 가
// stats: &mut IndexStats 를 받아 인덱싱하면서 계속 갱신합니다.
// 매번 새 구조체를 만들어 돌려주면 코드가 훨씬 번거로워집니다.

#[derive(Debug, Default, PartialEq)]
pub struct IndexStats {
    pub files_indexed: usize,
    pub symbols: usize,
}

pub fn record_file(stats: &mut IndexStats, symbol_count: usize) {
    stats.files_indexed += 1;
    stats.symbols += symbol_count;
}

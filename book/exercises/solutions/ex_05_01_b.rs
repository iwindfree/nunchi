// 정답: `&self` 를 `&mut self` 로 바꿉니다.
//
// 값을 읽기만 하는 메서드는 &self, 바꾸는 메서드는 &mut self 를 받습니다.
// 1.3장의 빌림 규칙이 메서드에도 그대로 적용됩니다.
//
// nunchi 의 실제 코드에서도 같은 구분이 보입니다.
//
//     pub fn count_nodes(&self) -> Result<i64>            읽기만 합니다
//     pub fn upsert_nodes(&mut self, ...) -> Result<usize>  데이터베이스를 바꿉니다

#[derive(Debug, Default)]
pub struct IndexStats {
    pub files_indexed: usize,
    pub symbols: usize,
}

impl IndexStats {
    pub fn add_file(&mut self, symbols: usize) {
        self.files_indexed += 1;
        self.symbols += symbols;
    }
}

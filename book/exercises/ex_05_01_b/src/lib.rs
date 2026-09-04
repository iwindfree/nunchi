// 아래 코드는 컴파일되지 않습니다.
//
// `add` 는 카운터를 늘려야 하는데 self 를 빌리는 방식이 잘못되었습니다.
// 오류 메시지를 읽고 고치십시오.
//
// 힌트: 값을 바꾸는 메서드는 self 를 어떻게 받아야 합니까? (1.3장)

#[derive(Debug, Default)]
pub struct IndexStats {
    pub files_indexed: usize,
    pub symbols: usize,
}

impl IndexStats {
    pub fn add_file(&self, symbols: usize) {
        // TODO: 이 두 줄에서 오류가 납니다
        self.files_indexed += 1;
        self.symbols += symbols;
    }
}

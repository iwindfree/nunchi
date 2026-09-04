// nunchi 의 SymbolTable 을 축소한 문제입니다.
//
// 심볼 이름 하나에 정의가 여러 개 있을 수 있습니다.
// HashMap<String, Vec<String>> 에 쌓아야 합니다.
//
// `insert` 를 완성하십시오. 이름이 처음 나오면 빈 목록을 만들고,
// 이미 있으면 거기에 덧붙입니다.
//
// 힌트: entry(...).or_default() 를 쓰면 두 경우를 한 줄로 처리합니다.

use std::collections::HashMap;

#[derive(Default)]
pub struct SymbolTable {
    by_name: HashMap<String, Vec<String>>,
}

impl SymbolTable {
    pub fn insert(&mut self, name: &str, id: String) {
        // TODO: 여기를 완성하십시오
        todo!()
    }

    pub fn candidates(&self, name: &str) -> &[String] {
        self.by_name.get(name).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

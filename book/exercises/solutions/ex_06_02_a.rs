// 정답
//
//     self.by_name.entry(name.to_string()).or_default().push(id);
//
// entry 는 "그 키의 자리" 를 돌려줍니다. 값이 있으면 그것을, 없으면
// 새로 만들 자리를 줍니다. or_default 는 없을 때 기본값(빈 Vec)을 넣습니다.
//
// 이렇게 쓰지 않으면 두 번 조회해야 합니다.
//
//     if !self.by_name.contains_key(name) {          한 번 조회
//         self.by_name.insert(name.to_string(), Vec::new());
//     }
//     self.by_name.get_mut(name).unwrap().push(id);  또 조회
//
// entry 는 한 번만 조회하므로 빠르고 unwrap 도 필요 없습니다.
//
// nunchi 의 resolve.rs 에 같은 코드가 있습니다.
//
//     self.by_name.entry(name.to_string()).or_default().push(id);

use std::collections::HashMap;

#[derive(Default)]
pub struct SymbolTable {
    by_name: HashMap<String, Vec<String>>,
}

impl SymbolTable {
    pub fn insert(&mut self, name: &str, id: String) {
        self.by_name.entry(name.to_string()).or_default().push(id);
    }

    pub fn candidates(&self, name: &str) -> &[String] {
        self.by_name.get(name).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

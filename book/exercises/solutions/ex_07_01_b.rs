// 정답
//
//     use std::collections::{HashMap, HashSet};
//
// 중괄호로 묶으면 같은 경로에서 여러 개를 한 번에 가져올 수 있습니다.
//
// use 는 이름을 그 자리로 가져오는 것일 뿐 실행되는 코드가 아닙니다.
// 그래서 성능에 영향이 없고, 순수하게 읽기 편하게 만드는 장치입니다.
//
// nunchi 의 파일 맨 위를 보면 같은 모양입니다.
//
//     use std::collections::{BTreeMap, HashMap};
//     use crate::model::*;

use std::collections::{HashMap, HashSet};

pub fn count_unique(names: &[&str]) -> usize {
    let mut set: HashSet<String> = HashSet::new();
    for n in names {
        set.insert(n.to_string());
    }
    set.len()
}

pub fn group(names: &[&str]) -> HashMap<usize, Vec<String>> {
    let mut map: HashMap<usize, Vec<String>> = HashMap::new();
    for n in names {
        map.entry(n.len()).or_default().push(n.to_string());
    }
    map
}

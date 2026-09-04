// 정답
//
// 트레이트는 "이런 메서드를 가진다"는 약속입니다. 구현은 각 타입이 합니다.
//
// `impl NodeStore for MemStore` 라고 읽습니다.
//   "MemStore 에 대해 NodeStore 를 구현한다"는 뜻입니다.
//
// describe 의 `<S: NodeStore>` 는 "NodeStore 를 구현한 어떤 타입 S 든"
// 이라는 뜻입니다. 이것이 트레이트의 값어치입니다. 함수를 고치지 않고
// 새 저장소를 추가할 수 있습니다.
//
// nunchi 에서는 이 장치로 SQLite 를 LadybugDB 같은 다른 저장소로
// 교체할 수 있게 해 두었습니다. 메서드를 여섯 개로 좁게 유지한 이유가
// 교체 비용을 하루 이내로 묶기 위해서입니다.

pub trait NodeStore {
    fn put(&mut self, id: String);
    fn count(&self) -> usize;
}

#[derive(Default)]
pub struct MemStore {
    ids: Vec<String>,
}

impl NodeStore for MemStore {
    fn put(&mut self, id: String) {
        self.ids.push(id);
    }

    fn count(&self) -> usize {
        self.ids.len()
    }
}

pub fn describe<S: NodeStore>(store: &S) -> String {
    format!("{} nodes", store.count())
}

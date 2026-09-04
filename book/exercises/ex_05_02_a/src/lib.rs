// nunchi 의 Store 트레이트를 축소한 문제입니다.
//
// 트레이트 `NodeStore` 를 정의하고 `MemStore` 에 구현하십시오.
//
// 트레이트에 있어야 하는 메서드:
//   fn put(&mut self, id: String)
//   fn count(&self) -> usize
//
// 트레이트를 쓰는 이유:
//   `describe` 함수가 어떤 저장소든 받을 수 있게 됩니다.
//   nunchi 에서 SQLite 를 다른 저장소로 바꿀 수 있게 만든 장치가 이것입니다.

// TODO: 여기에 NodeStore 트레이트를 정의하십시오

#[derive(Default)]
pub struct MemStore {
    ids: Vec<String>,
}

// TODO: 여기에 MemStore 에 대한 구현을 작성하십시오

/// 이 함수는 고치지 마십시오. 트레이트를 구현한 어떤 타입이든 받습니다.
pub fn describe<S: NodeStore>(store: &S) -> String {
    format!("{} nodes", store.count())
}

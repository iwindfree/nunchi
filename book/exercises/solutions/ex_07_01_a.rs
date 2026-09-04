// 정답: pub 을 다섯 곳에 붙입니다.
//
//   pub mod model          모듈을 공개합니다
//   pub mod store          모듈을 공개합니다
//   pub struct MemStore    타입을 공개합니다
//   pub fn put             메서드를 공개합니다
//   pub fn count           메서드를 공개합니다
//
// 필드 `ids` 는 공개하지 않았습니다. 밖에서 직접 만질 필요가 없기
// 때문입니다. 이것이 pub 의 값어치입니다. 무엇을 밖에 보여 줄지
// 선택해서 내부 구현을 바꿀 여지를 남깁니다.
//
// Rust 는 기본이 비공개입니다. 아무것도 붙이지 않으면 같은 모듈
// 안에서만 보입니다. 다른 언어에서 기본이 공개인 것과 반대입니다.
//
// nunchi 의 store 모듈이 같은 구조입니다. Store 트레이트와 SqliteStore 는
// 공개하지만 내부 연결 객체는 감춥니다.

pub mod model {
    #[derive(Debug, PartialEq)]
    pub struct NodeId(pub String);
}

pub mod store {
    use crate::model::NodeId;

    #[derive(Default)]
    pub struct MemStore {
        ids: Vec<NodeId>,
    }

    impl MemStore {
        pub fn put(&mut self, id: NodeId) {
            self.ids.push(id);
        }

        pub fn count(&self) -> usize {
            self.ids.len()
        }
    }
}

// 모듈을 공개하는 문제입니다.
//
// 아래 코드는 컴파일되지 않습니다. 테스트에서 model::NodeId 와
// store::MemStore 를 쓰려고 하는데 밖에서 보이지 않습니다.
//
// pub 을 붙여야 하는 자리를 찾아 고치십시오.
// 힌트: 모듈 자체와 그 안의 타입, 그리고 필드까지 각각 공개해야 합니다.

mod model {
    #[derive(Debug, PartialEq)]
    pub struct NodeId(pub String);
}

mod store {
    use crate::model::NodeId;

    #[derive(Default)]
    struct MemStore {
        ids: Vec<NodeId>,
    }

    impl MemStore {
        fn put(&mut self, id: NodeId) {
            self.ids.push(id);
        }

        fn count(&self) -> usize {
            self.ids.len()
        }
    }
}

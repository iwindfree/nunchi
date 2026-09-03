// 아래 코드는 컴파일은 되지만 경고가 납니다.
//
// Result 를 돌려주는 함수를 부르고 결과를 확인하지 않았습니다.
// 두 곳이 있는데 각각 다르게 고쳐야 합니다.
//
//   - save_index 의 실패는 중요합니다. 위로 올리십시오.
//   - touch_cache 의 실패는 무시해도 됩니다. 무시한다고 표시하십시오.
//
// 힌트: 2.2장의 "Result 를 무시하면 경고가 납니다" 를 보십시오.
//       cargo test 는 통과하지만 cargo build 에서 경고가 보입니다.

#[derive(Debug, Default)]
pub struct Store {
    pub saved: usize,
    pub touched: usize,
}

impl Store {
    pub fn save_index(&mut self) -> Result<(), String> {
        self.saved += 1;
        Ok(())
    }

    /// 캐시의 마지막 사용 시각을 갱신합니다. 실패해도 동작에 영향이 없습니다.
    pub fn touch_cache(&mut self) -> Result<(), String> {
        self.touched += 1;
        Ok(())
    }
}

pub fn run(store: &mut Store) -> Result<(), String> {
    store.save_index();   // TODO: 실패를 위로 올려야 합니다
    store.touch_cache();  // TODO: 무시한다고 표시해야 합니다
    Ok(())
}

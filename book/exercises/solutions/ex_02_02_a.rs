// 정답
//
//     store.save_index()?;
//     let _ = store.touch_cache();
//
// 두 가지를 구분하는 기준은 "이 실패가 뒤 동작에 영향을 주는가" 입니다.
//
// 인덱스 저장이 실패하면 그 뒤 작업이 의미가 없으므로 위로 올립니다.
// 캐시 시각 갱신은 실패해도 캐시가 동작하므로 무시합니다.
//
// let _ = 는 "일부러 무시했다" 는 표시입니다. 그냥 두면 컴파일러가
// 경고를 내는데, 그 경고는 "확인을 잊은 것이 아닌가" 라는 물음입니다.
// 표시를 해 두면 읽는 사람도 의도를 알 수 있습니다.
//
// nunchi 의 cache.rs 에 같은 형태가 있습니다.

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

    pub fn touch_cache(&mut self) -> Result<(), String> {
        self.touched += 1;
        Ok(())
    }
}

pub fn run(store: &mut Store) -> Result<(), String> {
    store.save_index()?;
    let _ = store.touch_cache();
    Ok(())
}

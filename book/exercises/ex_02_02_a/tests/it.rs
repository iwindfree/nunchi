use ex_02_02_a::{run, Store};

#[test]
fn runs_both_steps() {
    let mut store = Store::default();
    run(&mut store).unwrap();
    assert_eq!(store.saved, 1);
    assert_eq!(store.touched, 1);
}

/// 경고를 남긴 채로 두지 않았는지 소스를 직접 확인합니다.
#[test]
fn results_are_handled() {
    let source = include_str!("../src/lib.rs");
    let body = source
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        body.contains("store.save_index()?"),
        "save_index 의 실패는 ? 로 위로 올려야 합니다"
    );
    assert!(
        body.contains("let _ = store.touch_cache()"),
        "touch_cache 는 let _ = 로 무시한다고 표시해야 합니다"
    );
}

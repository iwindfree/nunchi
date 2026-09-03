use ex_04_01_c::{language, GUESS_CALLS};
use std::sync::atomic::Ordering;

#[test]
fn does_not_guess_when_declared() {
    GUESS_CALLS.store(0, Ordering::SeqCst);
    assert_eq!(language(Some("kotlin".into()), "a.rs"), "kotlin");
    assert_eq!(
        GUESS_CALLS.load(Ordering::SeqCst),
        0,
        "언어 이름이 있으면 추정 함수를 부르면 안 됩니다"
    );
}

#[test]
fn guesses_when_missing() {
    GUESS_CALLS.store(0, Ordering::SeqCst);
    assert_eq!(language(None, "a.java"), "java");
    assert_eq!(GUESS_CALLS.load(Ordering::SeqCst), 1);
}

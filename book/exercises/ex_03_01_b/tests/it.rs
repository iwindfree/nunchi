use ex_03_01_b::{resolve, Resolution};

#[test]
fn splits_by_candidate_count() {
    assert_eq!(resolve(0), Resolution::None);
    assert_eq!(resolve(1), Resolution::One(0.8));
    assert_eq!(resolve(2), Resolution::Many(0.4));
    assert_eq!(resolve(3), Resolution::Many(0.8 / 3.0));
    assert_eq!(resolve(4), Resolution::TooMany);
    assert_eq!(resolve(100), Resolution::TooMany);
}

use ex_01_03_c::{record_file, IndexStats};

#[test]
fn accumulates_across_files() {
    let mut stats = IndexStats::default();
    record_file(&mut stats, 12);
    record_file(&mut stats, 5);
    assert_eq!(stats, IndexStats { files_indexed: 2, symbols: 17 });
}

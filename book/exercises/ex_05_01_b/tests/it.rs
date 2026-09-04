use ex_05_01_b::IndexStats;

#[test]
fn accumulates_stats() {
    let mut stats = IndexStats::default();
    stats.add_file(10);
    stats.add_file(5);
    assert_eq!(stats.files_indexed, 2);
    assert_eq!(stats.symbols, 15);
}

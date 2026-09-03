use ex_00_02_a::sum_lines;

#[test]
fn sums_all_counts() {
    assert_eq!(sum_lines(&[1, 2, 3]), 6);
    assert_eq!(sum_lines(&[]), 0);
    assert_eq!(sum_lines(&[100]), 100);
}

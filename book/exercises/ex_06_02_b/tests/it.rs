use ex_06_02_b::tally;

#[test]
fn counts_occurrences() {
    let c = tally(&["save", "build", "save", "save"]);
    assert_eq!(c.get("save"), Some(&3));
    assert_eq!(c.get("build"), Some(&1));
    assert_eq!(c.get("missing"), None);
}

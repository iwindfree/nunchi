use ex_06_02_a::SymbolTable;

#[test]
fn accumulates_multiple_definitions() {
    let mut t = SymbolTable::default();
    t.insert("save", "sym:a#save".to_string());
    t.insert("save", "sym:b#save".to_string());
    t.insert("find", "sym:c#find".to_string());

    assert_eq!(t.candidates("save").len(), 2);
    assert_eq!(t.candidates("find").len(), 1);
    assert!(t.candidates("missing").is_empty());
}

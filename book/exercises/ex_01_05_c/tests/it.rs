use ex_01_05_c::{file_id, repo_id, symbol_id};

#[test]
fn builds_node_ids() {
    assert_eq!(repo_id("api"), "repo:api");
    assert_eq!(file_id("api", "src/A.java"), "file:api/src/A.java");
    assert_eq!(
        symbol_id("api", "src/A.java", "findOne"),
        "sym:api/src/A.java#findOne"
    );
}

#[test]
fn accepts_borrowed_strings() {
    let repo = String::from("web");
    // &String 이 &str 로 자동 변환되므로 그대로 넘길 수 있어야 합니다.
    assert_eq!(repo_id(&repo), "repo:web");
}

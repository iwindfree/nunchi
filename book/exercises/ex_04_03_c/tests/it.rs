use ex_04_03_c::repo_names;

#[test]
fn builds_name_to_path_map() {
    let paths = vec![
        "/home/me/dev/order-api".to_string(),
        "/home/me/dev/order-web".to_string(),
    ];
    let map = repo_names(&paths);
    assert_eq!(map.len(), 2);
    assert_eq!(map.get("order-api").map(String::as_str), Some("/home/me/dev/order-api"));
    assert_eq!(map.get("order-web").map(String::as_str), Some("/home/me/dev/order-web"));
}

#[test]
fn skips_paths_without_name() {
    let paths = vec!["/".to_string(), "/home/me/api".to_string()];
    let map = repo_names(&paths);
    assert_eq!(map.len(), 1);
    assert!(map.contains_key("api"));
}

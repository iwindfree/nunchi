use ex_04_02_b::{find_route, has_java_rule, RouteRule};

fn rules() -> Vec<RouteRule> {
    vec![
        RouteRule { lang: "java".into(), annotation: "GetMapping".into(), method: "GET".into() },
        RouteRule { lang: "java".into(), annotation: "PostMapping".into(), method: "POST".into() },
        RouteRule { lang: "python".into(), annotation: "get".into(), method: "GET".into() },
    ]
}

#[test]
fn detects_java_rules() {
    assert!(has_java_rule(&rules()));
    let only_python = vec![RouteRule {
        lang: "python".into(),
        annotation: "get".into(),
        method: "GET".into(),
    }];
    assert!(!has_java_rule(&only_python));
}

#[test]
fn finds_matching_rule() {
    let r = rules();
    let hit = find_route(&r, "java", "PostMapping").unwrap();
    assert_eq!(hit.method, "POST");
    assert!(find_route(&r, "java", "DeleteMapping").is_none());
    assert!(find_route(&r, "rust", "GetMapping").is_none());
}

#[test]
fn no_loops() {
    let source = include_str!("../src/lib.rs");
    let body: String = source
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!body.contains("for "), "반복문 대신 이터레이터를 쓰십시오");
}

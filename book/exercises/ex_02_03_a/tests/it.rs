use ex_02_03_a::{parse, Config};

#[test]
fn parses_config() {
    let c = parse("name = web", "3").unwrap();
    assert_eq!(c, Config { name: "web".into(), depth: 3 });
}

#[test]
fn reports_errors() {
    assert!(parse("no equals sign", "3").is_err());
    assert!(parse("name = web", "deep").is_err());
}

/// ? 로 줄였는지 확인합니다.
#[test]
fn uses_question_mark() {
    let source = include_str!("../src/lib.rs");
    let body: String = source
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let parse_fn = body.split("pub fn parse").nth(1).unwrap_or("");
    assert!(
        !parse_fn.contains("match "),
        "parse 안에 match 가 남아 있습니다. ? 로 바꾸십시오"
    );
    assert!(parse_fn.contains("?"), "? 연산자를 쓰십시오");
}

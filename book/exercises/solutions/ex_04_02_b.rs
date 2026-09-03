// 정답
//
// .any() 는 조건에 맞는 것이 하나라도 있으면 참입니다.
//   찾는 순간 멈추므로 나머지는 보지 않습니다.
//
// .find() 는 조건에 맞는 첫 값을 빌려서 돌려줍니다.
//   .iter() 를 썼으므로 Option<&RouteRule> 이 나옵니다.
//   규칙을 복사하지 않고 빌려 주므로 비용이 없습니다.
//
// 수명 표기 <'a> 가 필요한 이유:
//   rules 를 빌려서 그 안의 값을 빌려서 돌려줍니다. 참조를 여러 개 받으므로
//   반환값이 어디서 왔는지 알려 줘야 합니다(1.6장).
//   lang 과 annotation 에는 'a 가 없습니다. 반환값과 무관하기 때문입니다.
//
// nunchi 의 rules.rs 에 있는 route_for 가 정확히 이 형태입니다.

#[derive(Debug, PartialEq)]
pub struct RouteRule {
    pub lang: String,
    pub annotation: String,
    pub method: String,
}

pub fn has_java_rule(rules: &[RouteRule]) -> bool {
    rules.iter().any(|r| r.lang == "java")
}

pub fn find_route<'a>(
    rules: &'a [RouteRule],
    lang: &str,
    annotation: &str,
) -> Option<&'a RouteRule> {
    rules
        .iter()
        .find(|r| r.lang == lang && r.annotation == annotation)
}

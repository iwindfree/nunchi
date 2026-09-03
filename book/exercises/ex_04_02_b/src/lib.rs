// nunchi 의 규칙 조회를 단순하게 만든 것입니다.
//
// 두 함수를 이터레이터 메서드로 작성하십시오. 반복문을 쓰지 마십시오.
//
// 1. has_java_rule
//    규칙 중에 lang 이 "java" 인 것이 하나라도 있으면 참입니다.
//
// 2. find_route
//    lang 과 annotation 이 모두 맞는 첫 규칙을 찾아 빌려서 돌려줍니다.
//    없으면 None 입니다.
//
// 힌트: 4.2장의 "자주 쓰는 거두기" 표를 보십시오.

#[derive(Debug, PartialEq)]
pub struct RouteRule {
    pub lang: String,
    pub annotation: String,
    pub method: String,
}

pub fn has_java_rule(rules: &[RouteRule]) -> bool {
    todo!()
}

pub fn find_route<'a>(
    rules: &'a [RouteRule],
    lang: &str,
    annotation: &str,
) -> Option<&'a RouteRule> {
    todo!()
}

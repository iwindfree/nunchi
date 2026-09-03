// 아래 코드는 컴파일되지 않습니다. match 가 모든 경우를 다루지 않았습니다.
//
// 빠진 경우를 채우십시오. `_` 로 묶지 말고 명시적으로 적으십시오.
// 열거형에 값을 추가했을 때 컴파일러가 알려 주게 하기 위해서입니다(3.1장).
//
// 규칙: Calls 와 Injects 와 CallsApi 는 구조적 관계입니다(true).
//       ModifiedBy 와 AuthoredBy 는 이력 관계입니다(false).

#[derive(Debug, Clone, Copy)]
pub enum EdgeKind {
    Calls,
    Injects,
    CallsApi,
    ModifiedBy,
    AuthoredBy,
}

pub fn is_structural(kind: EdgeKind) -> bool {
    match kind {
        EdgeKind::Calls => true,
        EdgeKind::Injects => true,
        // TODO: 나머지 세 경우를 채우십시오
    }
}

// 정답
//
// Display 를 구현하면 두 가지가 공짜로 따라옵니다.
//   format!("{}", id) 와 id.to_string() 입니다.
//
// to_string 은 직접 만들지 않았는데도 생깁니다. 표준 라이브러리가
// "Display 를 구현한 모든 타입에 대해 to_string 을 제공한다" 고
// 미리 정해 두었기 때문입니다. 이것을 포괄 구현(blanket impl)이라 부릅니다.
//
// `Formatter<'_>` 의 '_ 는 수명 표기입니다(1.6장). 여기서는
// "컴파일러가 알아서 정하라"는 뜻이며 직접 적을 필요가 없습니다.
//
// nunchi 의 model.rs 에 같은 구현이 있습니다.

pub struct NodeId(pub String);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

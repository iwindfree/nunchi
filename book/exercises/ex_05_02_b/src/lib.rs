// 표준 트레이트를 구현하는 문제입니다.
//
// nunchi 의 NodeId 는 화면에 출력할 수 있어야 합니다.
// `Display` 트레이트를 구현하면 println!("{}", id) 가 됩니다.
//
// 구현해야 하는 것:
//   impl std::fmt::Display for NodeId
//   그 안에 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
//
// 힌트: 본문은 `f.write_str(&self.0)` 한 줄이면 됩니다.
//       self.0 은 튜플 구조체의 첫 번째 필드입니다(0.4장).

pub struct NodeId(pub String);

// TODO: 여기에 Display 구현을 작성하십시오

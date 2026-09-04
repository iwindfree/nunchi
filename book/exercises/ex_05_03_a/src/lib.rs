// From 을 구현하는 문제입니다.
//
// nunchi 에서 문자열을 NodeId 로 바꾸는 일이 자주 있습니다.
// `From<String> for NodeId` 를 구현하면 두 가지가 가능해집니다.
//
//   NodeId::from(s)
//   let id: NodeId = s.into();
//
// Into 는 직접 구현하지 않습니다. From 을 구현하면 Into 가 자동으로
// 따라옵니다. 그 이유는 5.3장에 있습니다.

#[derive(Debug, PartialEq)]
pub struct NodeId(pub String);

// TODO: 여기에 From<String> for NodeId 를 구현하십시오

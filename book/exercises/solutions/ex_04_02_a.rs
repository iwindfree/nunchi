// 정답: into_iter() 를 iter() 로 바꿉니다.
//
// .iter() 는 &String 을 꺼내 주고 원본은 그대로 둡니다.
// .len() 은 빌린 것으로도 부를 수 있으므로 문제가 없습니다.
//
// 세 가지를 구분하십시오(4.2장).
//   .iter()      &T 를 줍니다. 원본이 남습니다
//   .iter_mut()  &mut T 를 줍니다. 원본을 바꿉니다
//   .into_iter() T 를 줍니다. 원본이 사라집니다
//
// for x in v 는 into_iter() 를 부릅니다. for x in &v 는 iter() 입니다.
// 대부분의 경우 후자가 맞습니다.

pub fn summarize(names: Vec<String>) -> (usize, usize) {
    let total: usize = names.iter().map(|n| n.len()).sum();
    (names.len(), total)
}

// 정답
//
// extension 은 확장자가 없을 수도 있으므로 Option<String> 입니다.
//   `.map()` 이 Option 을 돌려주기 때문입니다(2.1장).
//
// line_count 는 개수이므로 usize 입니다.
//   `.count()` 가 usize 를 돌려줍니다. 개수와 크기에는 usize 를 씁니다(0.3장).

pub fn extension(path: &str) -> Option<String> {
    path.rsplit_once('.').map(|(_, ext)| ext.to_string())
}

pub fn line_count(text: &str) -> usize {
    text.lines().count()
}

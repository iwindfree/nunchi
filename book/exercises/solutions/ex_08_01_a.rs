// 정답
//
// 매크로 본문에서 $(...),+ 를 다시 쓰면 입력의 반복 횟수만큼 펼쳐집니다.
// match 의 팔을 만들 때는 쉼표 대신 세미콜론이나 아무것도 없이 반복할 수
// 있는데, 여기서는 match 팔이므로 $(...),+ 로 쉼표를 넣습니다.
//
// 매크로가 값을 하는 이유:
//   nunchi 에는 노드 18종과 엣지 19종이 있습니다. 각각에 as_str 과 parse 를
//   손으로 쓰면 74개 함수가 필요하고, 새 종류를 추가할 때마다 세 곳을
//   고쳐야 합니다. 매크로를 쓰면 한 줄만 추가하면 됩니다.
//
// 매크로의 대가:
//   오류 메시지가 어려워집니다. 매크로가 펼쳐진 뒤의 코드에서 오류가 나므로
//   원래 어느 줄이 문제인지 찾기 힘들 때가 있습니다.
//   그래서 반복이 많고 모양이 정확히 같을 때만 씁니다.

macro_rules! str_enum {
    ($name:ident { $($variant:ident => $s:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name { $($variant),+ }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $s),+ }
            }

            pub fn parse(s: &str) -> Option<Self> {
                match s { $($s => Some(Self::$variant),)+ _ => None }
            }
        }
    };
}

str_enum!(EdgeKind {
    Calls => "calls",
    Imports => "imports",
    Injects => "injects",
});

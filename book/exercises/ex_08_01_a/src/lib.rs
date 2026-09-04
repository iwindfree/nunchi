// nunchi 의 str_enum! 매크로를 축소한 문제입니다.
//
// 열거형을 정의할 때마다 as_str 과 parse 를 손으로 쓰는 것은 지겹고
// 빠뜨리기 쉽습니다. 매크로로 한 번에 만듭니다.
//
// 아래 매크로를 완성하십시오. 매크로가 만들어야 하는 것:
//   1. 열거형 자체
//   2. as_str(self) -> &'static str
//   3. parse(s: &str) -> Option<Self>
//
// 힌트:
//   $(...),+ 는 "하나 이상 반복" 을 뜻합니다.
//   본문에서도 $(...),+ 로 같은 횟수만큼 펼칩니다.
//   $(,)? 는 마지막에 쉼표가 있어도 되고 없어도 된다는 뜻입니다.

macro_rules! str_enum {
    ($name:ident { $($variant:ident => $s:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name { $($variant),+ }

        impl $name {
            // TODO: as_str 을 작성하십시오

            // TODO: parse 를 작성하십시오
        }
    };
}

str_enum!(EdgeKind {
    Calls => "calls",
    Imports => "imports",
    Injects => "injects",
});

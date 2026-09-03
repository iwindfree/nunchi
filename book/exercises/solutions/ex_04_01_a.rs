// 정답: 클로저 앞에 move 를 붙입니다.
//
//     check: Box::new(move |path| path.starts_with(&prefix)),
//
// move 가 없으면 클로저가 prefix 를 빌립니다. 그런데 이 클로저는 Filter
// 안에 담겨 함수 밖으로 나가고, prefix 는 함수가 끝나면서 사라집니다.
// 그러면 클로저가 없어진 값을 가리키게 되므로 컴파일러가 막습니다.
//
// move 를 붙이면 prefix 의 소유권이 클로저로 넘어갑니다. 이제 클로저가
// 어디로 가든 자기가 쓸 값을 가지고 다닙니다.
//
// nunchi 의 index.rs 에서 filter_entry 에 넘기는 클로저가 같은 이유로
// move 를 씁니다. 그 클로저는 walker 안에 저장되어 반복이 도는 내내
// 살아 있어야 합니다.

pub struct Filter {
    pub check: Box<dyn Fn(&str) -> bool>,
}

pub fn make_filter(prefix: String) -> Filter {
    Filter {
        check: Box::new(move |path| path.starts_with(&prefix)),
    }
}

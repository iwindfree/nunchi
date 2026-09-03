// 아래 코드는 컴파일되지 않습니다.
//
// 클로저를 만들어 구조체에 저장하려고 합니다. 그런데 클로저가 바깥 변수를
// 빌려서 가져가므로, 함수가 끝나면 그 빌림이 무효가 됩니다.
//
// 클로저가 만들어진 곳보다 오래 살아야 하므로 소유권을 가져가야 합니다.
// 한 낱말만 추가해서 고치십시오(4.1장).

pub struct Filter {
    pub check: Box<dyn Fn(&str) -> bool>,
}

pub fn make_filter(prefix: String) -> Filter {
    Filter {
        check: Box::new(|path| path.starts_with(&prefix)), // TODO
    }
}

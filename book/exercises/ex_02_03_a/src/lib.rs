// 아래 함수는 컴파일되고 테스트도 통과합니다.
//
// 다만 match 를 두 번 써서 오류를 처리하느라 실제 로직이 파묻혔습니다.
// ? 연산자를 써서 줄이십시오.
//
// 힌트: 2.3장의 첫 예제와 같은 형태입니다.

#[derive(Debug, PartialEq)]
pub struct Config {
    pub name: String,
    pub depth: usize,
}

fn read_name(raw: &str) -> Result<String, String> {
    raw.split_once('=')
        .map(|(_, v)| v.trim().to_string())
        .ok_or_else(|| "이름을 찾을 수 없습니다".to_string())
}

fn read_depth(raw: &str) -> Result<usize, String> {
    raw.trim().parse().map_err(|_| "깊이가 숫자가 아닙니다".to_string())
}

pub fn parse(name_line: &str, depth_line: &str) -> Result<Config, String> {
    let name = match read_name(name_line) {
        Ok(n) => n,
        Err(e) => return Err(e),
    };
    let depth = match read_depth(depth_line) {
        Ok(d) => d,
        Err(e) => return Err(e),
    };
    Ok(Config { name, depth })
}

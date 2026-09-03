// 정답
//
//     pub fn parse(name_line: &str, depth_line: &str) -> Result<Config, String> {
//         let name = read_name(name_line)?;
//         let depth = read_depth(depth_line)?;
//         Ok(Config { name, depth })
//     }
//
// 10줄이 3줄로 줄었고, 무엇을 하는 함수인지 한눈에 보입니다.
//
// ? 가 하는 일은 match 두 개와 정확히 같습니다.
//   성공하면 값을 꺼내고, 실패하면 그 오류로 함수를 끝냅니다.

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
    let name = read_name(name_line)?;
    let depth = read_depth(depth_line)?;
    Ok(Config { name, depth })
}

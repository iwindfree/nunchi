// 정답
//
// with_context 가 클로저를 받는 이유:
//   오류가 났을 때만 메시지를 만들기 위해서입니다. format! 은 힙 할당을
//   하므로, 성공하는 경우가 대부분인 코드에서 매번 만들면 낭비입니다.
//
// {err:#} 로 출력하면 맥락과 근본 원인이 함께 나옵니다.
//
//     설정 nunchi.toml 의 max_commits 값을 읽을 수 없습니다:
//     invalid digit found in string
//
// 맥락을 덧붙여도 원래 오류가 사라지지 않는다는 점이 중요합니다.

use anyhow::{Context, Result};

pub fn read_number(file: &str, line: &str, key: &str) -> Result<usize> {
    let (_, raw) = line
        .split_once('=')
        .with_context(|| format!("설정 {file} 의 {key} 줄에 등호가 없습니다"))?;
    let value = raw
        .trim()
        .parse::<usize>()
        .with_context(|| format!("설정 {file} 의 {key} 값을 읽을 수 없습니다"))?;
    Ok(value)
}

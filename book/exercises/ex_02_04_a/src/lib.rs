// 아래 함수는 동작하지만 오류 메시지가 쓸모없습니다.
//
// 실패하면 "invalid digit found in string" 같은 메시지만 나옵니다.
// 어떤 파일의 어떤 설정이 문제인지 알 수 없습니다.
//
// with_context 를 붙여서 맥락을 덧붙이십시오.
//
// 힌트: 2.4장의 "맥락을 덧붙입니다" 를 보십시오.
//       메시지에 file 과 key 가 모두 들어가야 합니다.

use anyhow::{Context, Result};

/// "key = value" 형식의 한 줄에서 숫자 설정을 읽습니다.
pub fn read_number(file: &str, line: &str, key: &str) -> Result<usize> {
    let (_, raw) = line
        .split_once('=')
        .context("등호가 없습니다")?; // TODO: 맥락을 덧붙이십시오
    let value = raw.trim().parse::<usize>()?; // TODO: 맥락을 덧붙이십시오
    Ok(value)
}

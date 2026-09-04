// 정답
//
// #[serde(default)] 은 그 필드가 없으면 Default::default() 를 씁니다.
//   Vec 의 기본값은 빈 목록입니다.
//
// #[serde(default = "함수이름")] 은 그 함수를 불러 기본값을 얻습니다.
//   usize 의 기본값은 0 이므로, 1000 을 원하면 함수를 따로 만들어야 합니다.
//   함수 이름을 문자열로 적는 것이 특이한데, 매크로가 코드를 생성할 때
//   이름으로 찾아 넣기 때문입니다.
//
// nunchi 의 config.rs 에 같은 패턴이 있습니다.
//
//     #[serde(default = "default_max_commits")]
//     pub max_commits: usize,

use serde::Deserialize;

#[derive(Deserialize)]
pub struct IndexConfig {
    pub languages: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default = "default_max_commits")]
    pub max_commits: usize,
}

fn default_max_commits() -> usize {
    1000
}

pub fn parse(text: &str) -> Result<IndexConfig, toml::de::Error> {
    toml::from_str(text)
}

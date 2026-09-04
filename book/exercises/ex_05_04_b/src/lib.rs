// serde 속성을 쓰는 문제입니다.
//
// nunchi 의 설정 파일은 일부 항목을 생략할 수 있습니다.
// 생략하면 기본값이 들어가야 합니다.
//
// 아래 구조체에 필요한 것을 추가하십시오.
//
//   1. Deserialize 를 derive 합니다
//   2. max_commits 는 생략 가능해야 하며 생략하면 1000 이 됩니다
//
// 힌트: #[serde(default = "함수이름")] 을 쓰고 그 함수를 정의합니다.
//       exclude 는 빈 목록이 기본이므로 #[serde(default)] 만으로 됩니다.

pub struct IndexConfig {
    pub languages: Vec<String>,
    pub exclude: Vec<String>,
    pub max_commits: usize,
}

pub fn parse(text: &str) -> Result<IndexConfig, toml::de::Error> {
    toml::from_str(text)
}

# 5.4 `#[derive]`와 serde 속성

> **선행 장**: [5.2 트레이트](05-2-traits.md)
> **연습문제**: 2개

`#[derive]`는 트레이트 구현을 자동으로 만들어 줍니다. nunchi에 53번 나옵니다.

## 반복되는 구현을 자동으로 만듭니다

`Clone`을 직접 구현하면 이렇습니다.

```rust
impl Clone for Span {
    fn clone(&self) -> Self {
        Span {
            start_line: self.start_line,
            end_line: self.end_line,
        }
    }
}
```

필드를 하나씩 복사하는 뻔한 코드입니다. 필드가 열 개면 열 줄이 됩니다.
`#[derive]`를 쓰면 한 줄입니다.

```rust
#[derive(Clone)]
pub struct Span {
    pub start_line: u32,
    pub end_line: u32,
}
```

컴파일러가 같은 코드를 자동으로 만듭니다.

## 자주 쓰는 것들

```rust
// crates/nunchi-core/src/model.rs 에서
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start_line: u32,
    pub end_line: u32,
}
```

| 이름 | 하는 일 | 조건 |
|---|---|---|
| `Debug` | `{:?}`로 출력합니다 | 모든 필드가 `Debug` |
| `Clone` | `.clone()`을 쓸 수 있습니다 | 모든 필드가 `Clone` |
| `Copy` | 복사가 자동으로 일어납니다 | 모든 필드가 `Copy`, `Clone` 필요 |
| `PartialEq` | `==`로 비교합니다 | 모든 필드가 `PartialEq` |
| `Eq` | 완전한 동등 비교입니다 | `PartialEq` 필요 |
| `Hash` | `HashMap`의 키로 쓸 수 있습니다 | 모든 필드가 `Hash` |
| `Default` | `Default::default()`를 만듭니다 | 모든 필드가 `Default` |
| `PartialOrd`, `Ord` | 크기를 비교하고 정렬합니다 | |

**조건이 중요합니다.** 필드 중 하나라도 해당 트레이트가 없으면 `derive`가
실패합니다. `String`이 들어 있는 구조체에 `Copy`를 붙일 수 없는 이유가
그것입니다([1.2장](01-2-move.md)).

## `Debug`는 거의 항상 붙입니다

오류를 찾을 때 값을 출력해 봐야 하기 때문입니다.

```rust
println!("{:?}", node);
```

`Debug`가 없으면 이것이 안 됩니다. 그리고 `assert_eq!`도 실패했을 때 값을
출력하므로 `Debug`가 필요합니다. 테스트를 쓰다가 `Debug`가 없다는 오류를
만나는 경우가 자주 있습니다.

## `Default`

기본값을 만듭니다. 숫자는 0, 문자열은 빈 문자열, `Option`은 `None`,
`Vec`은 빈 목록이 됩니다.

```rust
// crates/nunchi-core/src/index.rs 에서
#[derive(Debug, Default, Clone)]
pub struct IndexStats {
    pub repos: usize,
    pub files_seen: usize,
    // ... 스무 개가 넘습니다
}

let mut stats = IndexStats::default();     // 전부 0 으로 시작합니다
```

필드가 스무 개인데 하나씩 `0`으로 적을 필요가 없습니다.

기본값을 직접 정하고 싶으면 `Default`를 손으로 구현합니다.

```rust
// crates/nunchi-core/src/config.rs 에서
impl Default for RankWeights {
    fn default() -> Self {
        RankWeights {
            alpha_bm25: 0.7,
            beta_ppr: 0.5,
            gamma_recency: 0.3,
            delta_cochange: 0.4,
            epsilon_central: 0.2,
        }
    }
}
```

`derive`를 쓰면 전부 `0.0`이 되는데, 그러면 랭킹이 동작하지 않습니다. 그래서
직접 적었습니다.

## `..Default::default()`

필요한 필드만 적고 나머지는 기본값으로 채웁니다.

```rust
// crates/nunchi-cli/src/main.rs 에서
let opts = nunchi_core::pack::PackOptions {
    budget,
    weights: config.rank,
    synonyms: config.semantic.clone(),
    ..Default::default()
};
```

`PackOptions`에는 필드가 여섯 개인데 세 개만 적었습니다. 나머지는 `Default`가
채웁니다.

## serde 속성

`Serialize`와 `Deserialize`는 구조체를 JSON이나 TOML로 바꾸고 되돌립니다.
nunchi가 설정 파일을 읽고 MCP 응답을 만드는 데 씁니다.

`#[serde(...)]`로 세부 동작을 조정하며, nunchi에 47번 나옵니다.

| 속성 | 하는 일 |
|---|---|
| `#[serde(default)]` | 파일에 없으면 기본값을 씁니다 |
| `#[serde(rename = "ref")]` | 다른 이름으로 내보냅니다 |
| `#[serde(skip_serializing_if = "...")]` | 조건에 맞으면 내보내지 않습니다 |
| `#[serde(rename_all = "snake_case")]` | 이름 형식을 한꺼번에 바꿉니다 |

### `#[serde(default)]`가 중요한 이유

설정 파일에 `[index]` 절이 없어도 동작해야 합니다.

```toml
[solution]
name = "web"
repos = ["/dev/api"]
```

`#[serde(default)]`가 있으면 `index`가 기본값으로 채워집니다. 없으면 파싱이
실패합니다. **설정 파일이 최소한만 있어도 되게 만드는 장치입니다.**

### 이름을 바꾸는 이유

```rust
// crates/nunchi-core/src/pack.rs 에서
pub struct PackItem {
    pub tier: &'static str,
    #[serde(rename = "ref")]
    pub reference: String,
    // ...
}
```

JSON에서는 `ref`라는 이름을 쓰고 싶은데, `ref`는 Rust의 예약어라서 필드
이름으로 쓸 수 없습니다. 그래서 필드는 `reference`로 두고 내보낼 때만
`ref`로 바꿉니다.

### 없는 값을 빼는 이유

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub sig: Option<String>,
```

`None`이면 JSON에 넣지 않습니다. `"sig": null`이 들어가지 않으므로 응답이
짧아집니다.

**팩 응답에서 이것이 실제로 값을 합니다.** 토큰을 아끼는 것이 이 도구의
목적인데, 응답에 `null`이 잔뜩 들어가면 그만큼 낭비입니다.

## 연습문제

### 문제 1 [고치기]

```bash
cd book/exercises
cargo test -p ex_05_04_a
```

필요한 `derive`가 빠져서 컴파일되지 않는 문제입니다.

### 문제 2 [쓰기]

```bash
cargo test -p ex_05_04_b
```

serde 속성으로 JSON 출력을 조정하는 문제입니다.

## 정리

`#[derive]`는 뻔한 트레이트 구현을 자동으로 만듭니다. 모든 필드가 해당
트레이트를 갖고 있어야 합니다.

`Debug`는 거의 항상 붙입니다. 값을 출력하거나 `assert_eq!`를 쓰려면
필요합니다.

`Default`는 기본값을 만들며, 직접 정하고 싶으면 손으로 구현합니다.
`..Default::default()`로 나머지 필드를 채울 수 있습니다.

serde 속성 중 `#[serde(default)]`는 설정 파일이 최소한만 있어도 되게 만들고,
`skip_serializing_if`는 응답에서 없는 값을 빼서 토큰을 아낍니다.

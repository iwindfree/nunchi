# 8.1 `macro_rules!` 해부

> **선행 장**: [3.3 `let ... else`와 `matches!`](03-3-let-else.md), [5.4 `#[derive]`와 serde 속성](05-4-derive.md)
> **연습문제**: 2개

nunchi에 직접 만든 매크로가 **하나** 있습니다. 그 하나를 자세히 분석합니다.

## 매크로는 코드를 만듭니다

함수는 값을 받아 값을 돌려주지만, 매크로는 **코드를 받아 코드를 만듭니다.**
컴파일 전에 펼쳐집니다.

이미 여러 매크로를 쓰고 있습니다.

```rust
println!("{}", x);          // 매크로입니다
format!("{}/{}", a, b);     // 매크로입니다
vec![1, 2, 3];              // 매크로입니다
assert_eq!(a, b);           // 매크로입니다
matches!(x, Some(_));       // 매크로입니다
```

이름 뒤의 `!`가 매크로라는 표시입니다.

**함수로는 만들 수 없는 것을 만듭니다.** `println!`은 인자 개수가 정해져
있지 않고, `vec!`은 목록을 만들며, `matches!`는 패턴을 받습니다. 함수는 이런
일을 할 수 없습니다.

## nunchi의 `str_enum!`

문제 상황부터 봅니다. 열거형과 문자열을 오가는 코드를 계속 써야 했습니다.

```rust
pub enum NodeKind { Solution, Repo, File, /* ... 18개 */ }

impl NodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Solution => "solution",
            Self::Repo => "repo",
            Self::File => "file",
            // ... 18줄
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "solution" => Some(Self::Solution),
            "repo" => Some(Self::Repo),
            "file" => Some(Self::File),
            // ... 18줄
        }
    }
}

impl std::fmt::Display for NodeKind { /* ... */ }
```

`NodeKind`가 18개, `EdgeKind`가 19개, `Provenance`가 2개입니다. 같은 모양의
코드를 세 번 써야 하고, 값을 추가할 때마다 세 곳을 고쳐야 합니다.

**두 곳만 고치고 한 곳을 잊으면 오류 없이 틀린 채로 남습니다.** 그래서 매크로로 묶었습니다.

## 매크로 정의를 한 줄씩 읽습니다

```rust
// crates/nunchi-core/src/model.rs 에서
macro_rules! str_enum {
    ($name:ident { $($(#[$meta:meta])* $variant:ident => $s:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum $name { $($(#[$meta])* $variant),+ }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $s),+ }
            }
            pub fn parse(s: &str) -> Option<Self> {
                match s { $($s => Some(Self::$variant),)+ _ => None }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}
```

### 첫 줄: 받을 모양

```rust
($name:ident { $($(#[$meta:meta])* $variant:ident => $s:literal),+ $(,)? }) => {
```

`$`로 시작하는 것이 **받아 둘 자리**입니다. 뒤에 오는 이름이 받을 것의
종류입니다.

| 표기 | 받는 것 |
|---|---|
| `:ident` | 이름 (변수명, 타입명) |
| `:literal` | 값 (`"file"`, `3`) |
| `:meta` | 속성 (`#[...]` 안의 내용) |
| `:expr` | 식 |
| `:ty` | 타입 |

`$(...)`는 **반복**입니다.

- `$( ... ),+`는 쉼표로 나뉜 것이 하나 이상 온다는 뜻입니다.
- `$(,)?`는 마지막에 쉼표가 있어도 되고 없어도 된다는 뜻입니다.
- `$(#[$meta:meta])*`는 속성이 없어도 되고 여러 개여도 된다는 뜻입니다.

### 나머지: 만들 코드

`=>` 뒤가 실제로 만들어질 코드입니다. 받아 둔 자리를 그대로 씁니다.

```rust
pub enum $name { $($(#[$meta])* $variant),+ }
```

`$name`에 `NodeKind`가 들어가고, `$variant`가 반복되면서 값들이 채워집니다.

## 쓰는 쪽

```rust
str_enum!(NodeKind {
    Solution => "solution",
    Repo => "repo",
    File => "file",
    Module => "module",
    Symbol => "symbol",
    // ...
    Control => "control",
});
```

**값을 하나 추가하면 세 곳이 함께 늘어납니다.** 열거형 정의, `as_str`,
`parse`가 모두 자동으로 갱신됩니다.

## 실제로 겪은 오류

이 매크로를 처음 만들었을 때 `:meta`를 받는 부분이 없었습니다. 그래서 값에
주석을 달 수 없었습니다.

```rust
str_enum!(Provenance {
    /// tree-sitter 빠른 경로입니다
    Fast => "fast",
    /// SCIP 정밀 경로입니다
    Precise => "precise",
});
```

문서 주석은 사실 `#[doc = "..."]` 속성입니다. 매크로가 그것을 받을 준비가
안 되어 있어서 이런 오류가 났습니다.

```
error: no rules expected `#`
  |
  = note: outer doc comments expand to `#[doc = "..."]`,
          which is what this macro attempted to match
```

`$(#[$meta:meta])*`를 추가해서 고쳤습니다. **오류 메시지가 원인을 정확히
알려 준 경우입니다.**

## 매크로를 만들기 전에 생각할 것

매크로는 강력하지만 대가가 있습니다.

- 읽기 어렵습니다. 문법이 보통 Rust 코드와 다릅니다.
- 오류 메시지가 불친절해집니다. 펼쳐진 코드에서 오류가 나기 때문입니다.
- 편집기 지원이 약합니다. 자동 완성이 잘 안 됩니다.

**nunchi에 매크로가 하나뿐인 이유가 그것입니다.** 같은 모양이 세 번
반복되고, 값을 추가할 때 여러 곳을 함께 고쳐야 하는 상황이었기에 만들
가치가 있었습니다. 두 번 반복하는 정도였다면 그냥 두 번 썼을 것입니다.

## `#[derive]`와 무엇이 다른가

`#[derive]`도 매크로입니다. 다만 종류가 다릅니다.

| | `macro_rules!` | `#[derive]` |
|---|---|---|
| 부르는 방법 | `name!(...)` | 타입 위에 붙입니다 |
| 만드는 것 | 아무 코드나 | 트레이트 구현만 |
| 만드는 방법 | 패턴으로 | Rust 코드로 (별도 크레이트 필요) |

직접 `#[derive]`를 만들려면 절차적 매크로(procedural macro)를 써야 하고,
별도 크레이트가 필요합니다. nunchi에는 없습니다.

## 연습문제

### 문제 1 [읽기]

`str_enum!`이 만들어 내는 코드를 직접 적어 보십시오.

```rust
str_enum!(Direction {
    Out => "out",
    In => "in",
});
```

<details>
<summary>정답 보기</summary>

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction { Out, In }

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Out => "out",
            Self::In => "in",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "out" => Some(Self::Out),
            "in" => Some(Self::In),
            _ => None,
        }
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
```

값 두 개를 적었는데 코드 스물몇 줄이 만들어집니다. 값이 19개인
`EdgeKind`에서는 이 차이가 훨씬 큽니다.

</details>

### 문제 2 [쓰기]

```bash
cd book/exercises
cargo test -p ex_08_01_a
```

간단한 매크로를 만드는 문제입니다.

## 정리

매크로는 코드를 받아 코드를 만들며, 함수로는 할 수 없는 일을 합니다.
`$name:ident`처럼 받을 자리를 정의하고 `$(...)`로 반복을 표현합니다.

nunchi의 `str_enum!`은 열거형 정의와 문자열 변환 코드를 함께 만듭니다. 값을
추가할 때 여러 곳을 함께 고쳐야 하는 문제를 없앱니다.

매크로는 읽기 어렵고 오류 메시지가 불친절해지므로 같은 모양이 여러 번
반복될 때만 만드는 편이 낫습니다.

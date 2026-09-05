# 5. 어노테이션을 해석한다

> **필요한 문법**: [3.1 `match`](../rust/03-1-match.md),
> [3.3 `let ... else`와 `matches!`](../rust/03-3-let-else.md)

## 무엇을 하는 코드인가

앞 장에서 tree-sitter로 심볼과 호출을 뽑았습니다. 그런데 Spring 코드에서는
그것만으로 그래프가 거의 비어 있게 됩니다.

이유를 실제 코드로 보겠습니다.

```java
@RestController
@RequestMapping("/api/articles")
public class ArticleController {

    private final ArticleService articleService;   // 주입받습니다

    @GetMapping("/{slug}")
    public ArticleDto get(String slug) {
        return articleService.findBySlug(slug);
    }
}
```

`GET /api/articles/{slug}` 요청이 `get` 메서드로 온다는 사실이 코드 어디에도
**호출로 적혀 있지 않습니다.** `@GetMapping` 어노테이션에만 있습니다.

`articleService`가 어떻게 채워지는지도 마찬가지입니다. `new`를 부르는 곳이
없습니다. Spring이 실행 중에 넣어 줍니다.

이것을 처리하지 않으면 어떻게 되는지 실측했습니다. RealWorld 저장소를
인덱싱했을 때 해소되지 않은 호출의 상위가 이랬습니다.

```
save          45회    JPA 리포지터리라서 본문이 없습니다
build/builder 85회    Lombok 이 생성하는 코드입니다
assertThat    79회    AssertJ 이며 외부 라이브러리입니다
```

`framework.rs`가 이 문제를 다룹니다.

## 그림

```mermaid
flowchart TD
    A[구문 트리] --> B[클래스 선언을 만남]
    B --> C[modifiers 에서 어노테이션을 모음]
    C --> D{규칙에 있는 어노테이션인가}
    D -->|@RestController| E[Bean 으로 기록]
    D -->|@RequestMapping| F[경로 접두로 기록]
    D -->|@Entity| G[Entity 로 기록]
    E --> H[메서드로 내려감]
    F --> H
    H --> I{메서드 어노테이션}
    I -->|@GetMapping| J[Route 생성]
    I -->|@Select| K[SQL 에서 테이블 추출]
```

## 한 줄씩

### 왜 쿼리가 아니라 직접 순회하는가

앞 장에서는 tree-sitter 쿼리를 썼는데 여기서는 트리를 직접 순회합니다.

어노테이션과 선언의 관계를 쿼리로 표현하면 취약해집니다. 어노테이션이 여러
개 붙을 수 있고, 클래스와 메서드에 각각 붙으며, 언어마다 트리 모양이 다릅니다.
쿼리로 이 모든 조합을 적으면 길고 깨지기 쉬워집니다.

직접 순회하면 코드가 길어지는 대신 무슨 일이 일어나는지 눈에 보입니다.

### 어노테이션을 모읍니다

```rust
fn annotations_of<'a>(decl: Node<'a>, src: &'a [u8]) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    let mut cursor = decl.walk();
    for child in decl.children(&mut cursor) {
        // C# 어트리뷰트
        if child.kind() == "attribute_list" {
            // ...
            continue;
        }
        if child.kind() != "modifiers" {
            continue;
        }
        let mut mods = child.walk();
        for m in child.children(&mut mods) {
            match m.kind() {
                "marker_annotation" => {
                    if let Some(n) = m.child_by_field_name("name") {
                        out.push((text(n, src).to_string(), None));
                    }
                }
                "annotation" => {
                    let name = m.child_by_field_name("name").map(|n| text(n, src).to_string());
                    let args = m.child_by_field_name("arguments").map(|a| text(a, src).to_string());
                    if let Some(name) = name {
                        out.push((name, args));
                    }
                }
                _ => {}
            }
        }
    }
    out
}
```

`<'a>`가 이 코드베이스에 여섯 번뿐인 수명 표기 중 하나입니다
([1.6장](../rust/01-6-lifetimes.md)). 트리 노드와 소스 바이트가 같은 만큼
살아 있어야 한다는 뜻입니다. 노드는 소스를 가리키고 있으므로 소스가 먼저
사라지면 안 됩니다.

Java는 `marker_annotation`(인자 없음)과 `annotation`(인자 있음)으로 나뉩니다.
`@Entity`는 앞쪽이고 `@Table(name = "orders")`는 뒤쪽입니다.

C#은 트리 모양이 다릅니다. `[HttpGet("{id}")]`는 `attribute_list` 안의
`attribute`입니다. 그래서 앞부분에서 따로 처리합니다. 의미는 같은데 모양이
다른 경우입니다.

### 라우트를 만듭니다

```rust
if node.kind() == "method_declaration" {
    let handler = node.child_by_field_name("name")
        .map(|n| text(n, src).to_string())
        .unwrap_or_default();

    for (anno, args) in annotations_of(node, src) {
        let Some(rule) = rules.route_for(lang, &anno) else { continue };
        let args_text = args.unwrap_or_default();
        let method = rule.method_from_args_prefix.as_deref()
            .and_then(|prefix| method_from_args(&args_text, prefix))
            .unwrap_or_else(|| rule.method.clone());
        let raw = path_from_args(&args_text).unwrap_or_default();
        let suffix = normalize_route_path(&raw);
        let full = format!("{base_path}{suffix}");

        facts.routes.push(RouteFact {
            method,
            path: if full.is_empty() { "/".to_string() } else { full },
            raw_path: if raw.is_empty() { "/".into() } else { raw },
            handler: handler.clone(),
            span: span_of(node),
        });
    }
}
```

`let Some(rule) = rules.route_for(lang, &anno) else { continue };`가
핵심입니다. **어떤 어노테이션이 라우트인지 코드에 적혀 있지 않습니다.**
규칙 표에서 찾습니다.

`base_path`는 클래스에 붙은 `@RequestMapping("/api/articles")`에서 왔습니다.
메서드의 `@GetMapping("/{slug}")`와 합쳐 `/api/articles/{slug}`가 됩니다.

### 규칙을 데이터로 두는 이유

```rust
pub struct RouteRule {
    pub lang: String,
    pub annotation: String,
    pub method: String,
    pub method_from_args_prefix: Option<String>,
    pub receivers: Vec<String>,
    pub method_from_args_list: Option<String>,
}
```

`@GetMapping`을 Rust 코드에 직접 적으면, 새 프레임워크나 사내 관용구를
지원할 때마다 다시 빌드하고 다시 배포해야 합니다. 개발 장비와 업무 장비가
분리되어 있으면 그 왕복이 특히 번거롭습니다.

규칙 표로 빼면 설정 파일에 몇 줄 추가하는 것으로 끝납니다.

```toml
[[framework.route]]
lang = "java"
annotation = "InternalEndpoint"
method = "POST"
```

이 틀에 Spring, NestJS, Micronaut, ASP.NET, Ktor, FastAPI, Flask가 모두
들어갑니다.

내장 기본 규칙도 Rust 코드가 아니라 `crates/nunchi-core/rules/`의 TOML
파일들에 있습니다. 언어별로 나뉘어 있어서 새 언어를 지원할 때 한 파일만
보면 됩니다. `include_str!`이 컴파일 시점에 넣으므로 배포물은 여전히 실행 파일
하나입니다. 앞 장의 tree-sitter 쿼리를 `.scm` 파일에 둔 것과 같은 방식입니다.

처음에는 Rust 코드였는데 이렇게 옮겼습니다. 규칙 하나를 더하는 일은 "이
어노테이션은 이 HTTP 메서드다"라는 사실을 적는 것뿐인데, 코드로 두면
`"java".into()`와 `["a", "b"].iter().map(|s| s.to_string()).collect()` 같은
관용구를 알아야 했습니다. Spring을 아는 사람이 Rust를 몰라서 규칙을 추가하지
못하는 상황이 생깁니다.

대신 필드 이름을 잘못 적어도 컴파일이 됩니다. 그래서 `builtin_rules_parse`
테스트로 막았습니다. 쿼리 파일에서 겪은 문제와 그 해결책이 같습니다.

### 파이썬은 데코레이터입니다

```rust
fn python_decorators<'a>(node: Node<'a>, src: &'a [u8]) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    let Some(parent) = node.parent() else { return out };
    if parent.kind() != "decorated_definition" {
        return out;
    }
    // ...
    let (receiver, name) = match head.rsplit_once('.') {
        Some((r, n)) => (r.to_string(), n.to_string()),
        None => (String::new(), head.to_string()),
    };
    out.push((receiver, name.trim().to_string(), args));
    out
}
```

파이썬은 `@app.get("/orders")` 형태입니다. 수신자(`app`)와 이름(`get`)이
점으로 나뉩니다.

이름만 보면 문제가 생깁니다. `@cache.get`도 `get`이므로 라우트로 인식됩니다.
그래서 규칙에 허용 수신자 목록을 둡니다.

```rust
pub fn route_for_receiver(&self, lang: &str, receiver: &str, name: &str) -> Option<&RouteRule> {
    self.route.iter().find(|r| {
        Self::lang_matches(&r.lang, lang)
            && r.annotation == name
            && (r.receivers.is_empty()
                || r.receivers.iter().any(|x| x.eq_ignore_ascii_case(receiver)))
    })
}
```

`receivers`가 비어 있으면 수신자를 따지지 않습니다. Java의 `@GetMapping`은
수신자가 없으므로 빈 목록입니다.

### 경로 표기를 통일합니다

```rust
pub fn normalize_route_path(raw: &str) -> String {
    let trimmed = raw.trim().trim_matches(['"', '\'', '`']);
    let mut out = String::with_capacity(trimmed.len());
    let mut chars = trimmed.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '$' if chars.peek() == Some(&'{') => {
                chars.next();
                for c in chars.by_ref() {
                    if c == '}' { break; }
                }
                out.push_str("{}");
            }
            '{' => {
                for c in chars.by_ref() {
                    if c == '}' { break; }
                }
                out.push_str("{}");
            }
            ':' if out.ends_with('/') => {
                while chars.peek().is_some_and(|c| *c != '/') {
                    chars.next();
                }
                out.push_str("{}");
            }
            _ => out.push(c),
        }
    }
    // 선행 슬래시를 보장하고 후행 슬래시를 없앱니다
    // ...
}
```

이 함수가 프런트엔드와 백엔드를 잇는 핵심입니다.

| 표기 | 쓰는 곳 | 정규화 결과 |
|---|---|---|
| `/api/orders/{id}` | Spring | `/api/orders/{}` |
| `/api/orders/:id` | Express, react-router | `/api/orders/{}` |
| `` `/api/orders/${id}` `` | JavaScript 템플릿 | `/api/orders/{}` |

세 표기가 같은 문자열이 되므로 연결 판정은 단순 비교로 끝납니다.

`chars.peek()`는 다음 글자를 미리 보되 소비하지 않습니다. `$` 다음이 `{`인지
확인해야 하는데, 아니라면 `$`를 그대로 두어야 하므로 필요합니다.

### 라우트 정의와 클라이언트 호출을 구분합니다

여기서 실제로 겪은 문제를 다룹니다.

```rust
fn api_call_of(
    call: Node,
    src: &[u8],
    clients: &[&HttpClientRule],
    syntax: &CallSyntax,
) -> Option<ApiCallFact> {
    let (receiver, callee) = callee_of(call, src, syntax)?;

    let (method, url_arg) = clients.iter().find_map(|rule| -> Option<(String, usize)> {
        match receiver.as_deref() {
            None => (rule.callee.as_deref() == Some(callee.as_str()))
                .then(|| (/* fetch(...) */)),
            Some(recv) => {
                if rule.exclude_receivers.iter().any(|x| x.eq_ignore_ascii_case(recv)) {
                    return None;
                }
                // ...
            }
        }
    })?;

    let args = call.child_by_field_name("arguments")?;

    if has_function_argument(args, syntax) {
        return None;
    }
    // ...
}
```

`axios.post('/api/orders', body)`는 클라이언트 호출입니다. 그런데
`this.post('/users', handler)`는 목 서버가 라우트를 **등록**하는 코드이며
호출이 아닙니다.

처음에는 이 둘을 구분하지 못했습니다. 탐지된 API 호출 21건 중 16건이 목
서버의 라우트 정의였습니다. 목 서버가 같은 API 명세를 그대로 반영하고
있었으므로 백엔드와 잘 연결되었고, 그래서 지표가 95%로 좋게 나왔습니다.
**잘못된 이유로 높게 나온 지표였습니다.**

두 가지로 막았습니다.

첫째, `exclude_receivers`에 `this`, `app`, `router`, `server`를 넣었습니다.

둘째, `has_function_argument`가 구조로 판정합니다. 인자에 함수나 화살표
함수가 있으면 핸들러를 등록하는 코드입니다. 이 판정은 프레임워크와 무관하게
동작합니다.

수정한 뒤 실제 클라이언트 호출은 4건이었고 연결도 4건이었습니다.

### 언어마다 구문 트리의 이름이 다릅니다

이 탐지기에서 실제로 겪은 결함입니다. 처음에는 이렇게 썼습니다.

```rust
if node.kind() == "call_expression" {
    // 호출을 발견했다
}
```

TypeScript에서는 잘 동작했습니다. 그런데 Python과 C# 규칙을 설정에 넣어
두었는데도 결과가 하나도 나오지 않았습니다.

원인은 **`call_expression`이라는 이름이 언어마다 다르기 때문**이었습니다.
[4장](04-parse.md)에서 Java와 TypeScript의 트리를 나란히 놓고 본 그 차이입니다.
각 언어의 파서에 직접 물어보니 이랬습니다.

| 언어 | 호출식 노드 이름 |
|---|---|
| TypeScript, JavaScript | `call_expression` |
| Python | `call` |
| Java | `method_invocation` |
| C# | `invocation_expression` |

이름만 다른 것이 아니라 구조도 다릅니다. TypeScript는 `function` 필드 아래에
수신자와 메서드가 있는데, Java는 호출식 자체가 `object`와 `name` 필드를
직접 갖습니다. C#은 실인자가 `argument` 노드로 한 겹 더 감싸여 있습니다.

그래서 언어별 이름을 표로 만들었습니다.

```toml
[[lang_syntax]]
lang = "java"
call = ["method_invocation"]
member = ["method_invocation"]
receiver_field = "object"
method_field = "name"
string = ["string_literal"]
lambda = ["lambda_expression"]
member_is_call = true
```

이 표도 처음에는 Rust 코드였는데 `rules/builtin.syntax.toml`로 옮겼습니다.
프레임워크 규칙과 마찬가지로 절차가 아니라 값이기 때문입니다. **규칙 파일에는
값만 두고 코드에는 절차만 둔다**는 경계가 이렇게 생겼습니다.

**이 결함이 오래 남아 있던 이유**가 중요합니다. `nunchi rules`를 실행하면
Python과 C# 규칙이 목록에 나옵니다. 규칙이 등록되어 있으니 동작한다고
믿기 쉽습니다. 그러나 규칙이 있다는 것과 그 규칙이 쓰인다는 것은 다릅니다.

그래서 네 언어를 모두 확인하는 테스트를 넣었습니다.

```rust
#[test]
fn detects_api_calls_in_every_supported_language() {
    // 언어마다 최소 한 건씩 실제로 탐지되는지 확인합니다
}
```

언어를 추가할 때 `CallSyntax`에 항목을 넣는 것을 잊으면 이 테스트가
실패합니다.

### 정적으로 알 수 없는 경로

```rust
pub fn has_dynamic_segment(raw: &str) -> bool {
    let trimmed = raw.trim().trim_matches(['"', '\'', '`']);
    let bytes = trimmed.as_bytes();
    let mut i = 0;
    while let Some(pos) = trimmed[i..].find("${") {
        let start = i + pos;
        if start == 0 || bytes[start - 1] != b'/' {
            return true;
        }
        // ...
    }
    false
}
```

`` `/users${isRegister ? '' : '/login'}` ``은 조건에 따라 `/users`가 되기도
하고 `/users/login`이 되기도 합니다. 어떤 엔드포인트인지 확정할 수 없습니다.

치환이 경로 세그먼트 전체를 차지하면(`/orders/${id}`) 파라미터로 볼 수
있지만, 세그먼트 중간에 끼어 있으면 알 수 없습니다. 이 함수가 그것을
구분합니다.

이런 경우를 연결 실패로 세면 지표가 왜곡되므로 `dynamic`으로 따로
집계합니다.

### SQL에서 테이블을 뽑습니다

```rust
pub fn tables_in_sql(sql: &str) -> Vec<(String, String)> {
    const KEYWORDS: &[(&str, &str)] = &[
        ("from", "select"),
        ("join", "select"),
        ("into", "insert"),
        ("update", "update"),
        ("delete from", "delete"),
    ];
    // FROM/JOIN/INTO/UPDATE 뒤의 식별자만 집습니다
}
```

SQL 파서를 붙이지 않았습니다. MyBatis SQL에는 `#{param}`이나 `<if>` 같은
템플릿 조각이 섞여 있어서 정식 파싱이 자주 실패하기 때문입니다.

키워드 뒤의 식별자만 추출하는 편이 견고합니다. 스키마 접두(`dbo.orders`)는
제거하고, SQL 예약어와 두 글자 미만 토큰은 제외합니다.

## 왜 이렇게 썼는가

### 왜 이 계층이 없으면 안 되는가

이 계층을 넣기 전과 후를 비교하면 이렇습니다.

| | 전 | 후 |
|---|---|---|
| 라우트 | 0 | 19 |
| Bean | 0 | 32 |
| 주입 | 0 | 48 |
| 교차 저장소 연결 | 0 | 4 |

라우트가 없으면 프런트엔드와 백엔드를 이을 방법이 없습니다. 그것이 이
프로젝트의 존재 이유이므로 이 계층이 빠지면 남는 것이 별로 없습니다.

## 정리

Spring에서는 호출 관계가 어노테이션에만 있으므로 tree-sitter만으로는
그래프가 비어 있게 됩니다. `framework.rs`가 어노테이션을 해석해 `Route`,
`Bean`, `Entity` 노드를 만듭니다.

어떤 어노테이션이 무엇을 뜻하는지는 코드가 아니라 규칙 표에 있습니다.
설정 파일로 확장할 수 있게 하기 위해서입니다.

경로 표기 세 가지를 하나로 정규화하는 것이 프런트엔드와 백엔드를 잇는
핵심입니다.

라우트를 정의하는 코드를 클라이언트 호출로 오인했던 문제는
`exclude_receivers`와 "인자에 함수가 있으면 등록"이라는 구조적 판정으로
막았습니다.

호출식 노드 이름이 언어마다 다르다는 사실을 놓쳐서 Python과 C#의 규칙이
동작하지 않았던 적도 있습니다. 규칙이 등록되어 있다는 사실만으로는 그 규칙이
쓰이고 있다고 말할 수 없습니다.

다음 장에서는 이렇게 만든 노드를 저장하는 부분을 봅니다.

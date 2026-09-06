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

### 메서드가 어디에 적혀 있는가

경로만큼 중요한 것이 HTTP 메서드입니다. 그런데 **메서드를 어디에 적는지가
라이브러리마다 다릅니다.**

처음에는 호출된 메서드 이름이 곧 HTTP 메서드라고 보았습니다.
`axios.post(...)`는 POST이고 `rest.getForObject(...)`는 GET입니다. 실제
코드에서 흔한 스무 가지 형태로 재어 보니 열한 가지만 잡혔습니다.

가장 나빴던 것이 이것입니다.

```javascript
fetch("/api/orders", { method: "POST" });
```

**프런트엔드에서 GET이 아닌 호출은 거의 전부 이 형태입니다.** 함수 이름은
언제나 `fetch`이고 메서드는 두 번째 인자에 있습니다. 그것을 읽지 않으면
전부 GET으로 기록됩니다.

놓치는 것보다 나쁩니다. 놓치면 그 호출이 그래프에 없을 뿐이지만, 틀린
메서드로 기록하면 **엉뚱한 라우트에 이어집니다.** `POST /api/orders`를
불렀는데 `GET /api/orders`를 처리하는 핸들러로 연결됩니다.

그래서 메서드를 어디서 읽을지도 규칙에 적습니다.

| 필드 | 어떤 형태 |
|---|---|
| `method_option` | `fetch(url, { method: "POST" })` |
| `url_option` | `axios({ method: "post", url: "/api/orders" })` |
| `method_arg` | `rest.exchange(url, HttpMethod.POST, entity, X.class)` |
| `method_from_receiver` | `webClient.get().uri("/api/orders")` |

읽는 절차는 정확한 자리부터 봅니다. 코드에 적힌 메서드가 규칙의 기본값을
이깁니다.

```rust
fn method_of(...) -> Option<String> {
    // 1. 설정 객체에 적혀 있으면 그것이 가장 정확하다. `fetch(url, {method})`
    if let Some(key) = &rule.method_option {
        if let Some(node) = option_value(args, key, src, syntax) {
            if let Some(method) = http_method_of(node, src, syntax) {
                return Some(method);
            }
        }
    }
    // 2. 인자로 넘기는 형태. `rest.exchange(url, HttpMethod.POST, ...)`
    //
    // 이 자리를 읽지 못하면 메서드를 알 수 없다. 아래로 흘려보내면 호출된
    // 메서드 이름인 `EXCHANGE`가 HTTP 메서드로 들어가므로 그 호출을 버린다.
    if let Some(index) = rule.method_arg {
        return argument_at(args, index, syntax).and_then(|n| http_method_of(n, src, syntax));
    }
    // 3. 체이닝. `webClient.get().uri(...)`
    if !rule.method_from_receiver.is_empty() {
        return receiver_call_method(receiver?, src, syntax).map(|m| m.to_ascii_uppercase());
    }
    // 4. 규칙에 고정된 값
    if let Some(method) = &rule.method {
        return Some(method.clone());
    }
    // 5. 호출된 메서드 이름이 곧 HTTP 메서드다.
    Some(callee.to_ascii_lowercase().trim_end_matches("async").to_ascii_uppercase())
}
```

**모르면 버립니다.** 2번에서 `rest.exchange(url, verb, ...)`처럼 메서드가
변수면 값을 알 수 없습니다. 여기서 아래로 흘려보내면 5번이 호출된 메서드
이름을 써서 `EXCHANGE`라는 HTTP 메서드가 그래프에 들어갑니다.

읽어 낸 값도 검사합니다.

```rust
let method = raw.trim().to_ascii_uppercase();
const KNOWN: [&str; 7] = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
KNOWN.contains(&method.as_str()).then_some(method)
```

대문자만 씌워 통과시키면 `HttpMethod.valueOf(x)` 같은 것이 `VALUEOF(X)`가
되어 들어갑니다.

C#은 동사마다 규칙을 나누었습니다. 이름에서 `Async`만 벗기는 방식으로는
`PostAsJsonAsync`가 `POSTASJSON`이 됩니다. 확장 메서드가 흔한 언어라 이름을
그대로 적어 두는 편이 안전합니다.

이 스무 가지 표는 테스트로 남아 있습니다.

```rust
#[test]
fn detects_the_shapes_real_code_uses() {
```

형태 하나를 놓치면 그 프로젝트의 호출이 통째로 빠지는데, **오류가 나지 않고
결과만 조용히 빕니다.** 표로 두면 새 형태를 더할 때 한 줄만 쓰면 되고,
잡으면 안 되는 것들도 같은 표에서 함께 확인합니다.

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

### 리터럴이 아닌 URL을 읽습니다

경로가 문자열 리터럴로 그대로 적혀 있는 경우는 많지 않습니다. 상수를 이어
붙이거나 변수를 끼워 넣습니다.

```java
private static final String BASE = "/api/orders";
rest.getForObject(BASE + "/" + id, OrderDto.class);
```

이 한 줄에 세 가지가 섞여 있습니다. 값을 아는 이름(`BASE`), 리터럴(`"/"`),
값을 모르는 변수(`id`)입니다. **이 구조를 문자열 하나로 뭉치지 않고 조각인
채로 들고 다니는 것**이 이 절의 핵심입니다.

```rust
{{#include ../../../crates/nunchi-core/src/framework.rs:url_part}}
```

`Named`가 왜 필요한지는 뒤에서 설명합니다. 위 코드는 이렇게 됩니다.

```rust
[Literal("/api/orders"), Literal("/"), Unknown]
```

### 조각으로 두면 판단이 짧아집니다

경로를 다루면서 물어야 하는 것이 네 가지입니다. 조각이 남아 있으면 그 질문이
코드에 그대로 옮겨집니다.

```rust
{{#include ../../../crates/nunchi-core/src/framework.rs:url_template}}
```

`is_dynamic`만 설명이 필요합니다. **값을 모르는 자리가 경로 세그먼트 하나를
통째로 차지하면 파라미터로 봅니다.** Spring의 `/{id}`가 정규화된 결과와 같은
문자열이 되므로 라우트에 연결할 수 있기 때문입니다.

```
/api/orders/{}      ← 슬래시 사이를 통째로 차지한다. 연결 가능
/api/orders{}       ← 세그먼트 중간에 끼어든다. 어느 경로인지 모른다
```

`"/api/orders" + suffix`가 뒤쪽입니다. `suffix`에 무엇이 오느냐에 따라
`/api/orders/x`도 되고 `/api/ordersx`도 됩니다.

### 조각을 모으는 곳

```rust
{{#include ../../../crates/nunchi-core/src/framework.rs:collect_parts}}
```

노드 종류를 그대로 조각 종류로 옮기는 것이 전부입니다. 연결식에서 재귀로
내려가는 부분이 중요합니다. `"/api/articles/" + slug + "/comments"`는 트리에서
왼쪽으로 중첩되므로, 왼쪽을 한 겹만 보면 최상위의 왼쪽이 또 연결식이라
리터럴을 찾지 못하고 **호출을 통째로 놓칩니다.** 실제로 그렇게 만들었다가
고쳤습니다.

문자열 리터럴도 그냥 넣지 않고 한 번 더 나눕니다. 리터럴이라고 전부 고정된
것은 아니기 때문입니다.

```
`/api/orders/${id}`     자바스크립트 템플릿
f"/api/orders/{id}"     파이썬 f-문자열
$"/api/orders/{id}"     C# 보간
```

셋 다 문자열 노드 하나인데 안에 치환이 들어 있습니다. `literal_parts`가
그것을 리터럴과 `Unknown`으로 갈라 놓습니다.

### 다른 파일의 상수는 2패스에서 채웁니다

경로 상수를 한곳에 모아 두는 관례가 흔합니다.

```java
// ApiPaths.java
public static final String ORDERS = "/api/orders";

// OrderGateway.java
rest.getForObject(ApiPaths.ORDERS, List.class);
```

이 장의 코드는 파일 하나만 봅니다. 다른 파일의 선언을 알 수 없으므로 값을
포기하는 대신 **이름을 남깁니다.** 그것이 `Named`입니다.

```rust
[Named("ApiPaths.ORDERS")]
```

[7장](07-resolve.md)에서 본 인덱싱 2패스가 전체 파일의 상수를 합쳐 이 자리를
채웁니다. 참조를 해소할 때 두 번 도는 이유와 같습니다. 다른 파일을 봐야 알 수
있는 것은 모든 파일을 읽은 다음에야 처리할 수 있습니다.

```rust
[Named("ApiPaths.ORDERS")]  →  fill  →  [Literal("/api/orders")]
```

표에 없으면 `Unknown`이 됩니다. 끝내 알 수 없는 값이라는 뜻입니다.

같은 이름이 여러 파일에 다른 값으로 있으면 값을 확정하지 않습니다. `BASE`나
`API_URL`은 흔해서 충돌하기 쉽습니다. 대신 클래스 이름을 붙인 키도 함께 넣어
두므로 `ApiPaths.ORDERS`처럼 한정해서 참조하면 정확히 해소됩니다.

### 마지막에 한 번만 문자열로 만듭니다

```rust
{{#include ../../../crates/nunchi-core/src/framework.rs:render}}
```

**이 프로젝트에서 `{}` 표기를 만들어 내는 유일한 자리입니다.** 조립하는
동안에는 그 기호가 코드에 나오지 않습니다.

처음에는 조각을 문자열에 기호로 심어 두고 단계마다 다시 파싱했습니다.
`"${ApiPaths.ORDERS}/${}"` 같은 문자열을 만들어 놓고 `${`를 찾아 가며
읽는 방식이었습니다. 그러자 기호가 세 겹으로 겹쳤습니다. Spring이 쓰는
`{id}`, 자바스크립트가 쓰는 `${id}`, 우리가 만든 `${}`가 같은 문자열
공간에서 돌아다녔고, 코드만 봐서는 어느 것이 입력이고 어느 것이 중간
표시인지 구분되지 않았습니다.

조각을 타입으로 두면 그 문제가 사라집니다. 심을 기호가 없고 파싱할 일도
없습니다.

실측하면 전형적인 아홉 가지 형태 중 여섯 가지를 읽습니다. 남은 셋은 설정에서
주입받는 값과 빌더 체이닝, 함수 반환값이며 정적으로는 알 수 없습니다.

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

//! 프레임워크 규칙 — 설정으로 확장 가능한 의미론 (docs/DESIGN.md 4·5절)
//!
//! Spring의 `@GetMapping`이나 React의 `axios.get`을 Rust 코드에 박아두면,
//! 새 프레임워크나 사내 관용구를 지원할 때마다 재빌드·재배포가 필요하다.
//! 회사 장비가 별도 머신인 이 프로젝트에서는 그 루프가 특히 비싸다(docs/CONTRIBUTING.md 개발 환경).
//!
//! 그래서 규칙을 데이터로 뺀다. 내장 기본값이 있고, `nunchi.toml`의 `[framework]`가
//! 그 위에 **덧붙는다**(`replace_defaults = true`면 대체).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkRules {
    /// true면 내장 기본 규칙을 버리고 사용자 규칙만 쓴다.
    #[serde(default)]
    pub replace_defaults: bool,
    #[serde(default)]
    pub route: Vec<RouteRule>,
    #[serde(default)]
    pub base_path: Vec<BasePathRule>,
    #[serde(default)]
    pub bean: Vec<BeanRule>,
    #[serde(default)]
    pub inject: Vec<InjectRule>,
    #[serde(default)]
    pub http_client: Vec<HttpClientRule>,
    /// 영속 계층 — 엔티티·테이블·SQL 매퍼
    #[serde(default)]
    pub persistence: Vec<PersistenceRule>,
    /// 언어별 구문 트리 노드 이름. 프레임워크 규칙이 아니라 언어 문법에
    /// 종속된 값이므로 사용자가 건드릴 일이 거의 없다.
    #[serde(default)]
    pub lang_syntax: Vec<LangSyntax>,
}

impl Default for FrameworkRules {
    fn default() -> Self {
        FrameworkRules {
            replace_defaults: false,
            route: Vec::new(),
            base_path: Vec::new(),
            bean: Vec::new(),
            inject: Vec::new(),
            http_client: Vec::new(),
            persistence: Vec::new(),
            lang_syntax: Vec::new(),
        }
    }
}

/// 한 언어의 구문 트리에서 호출식을 찾고 읽는 데 필요한 노드 이름들.
///
/// tree-sitter 문법이 언어마다 따로 만들어져서 같은 뜻인데 이름이 다르다.
/// 메서드 호출이 TypeScript에서는 `call_expression`이지만 Java는
/// `method_invocation`이다. 값은 `rules/builtin.syntax.toml`에 있다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangSyntax {
    pub lang: String,
    /// 호출식 노드 이름
    pub call: Vec<String>,
    /// 수신자와 메서드 이름을 담은 노드 이름
    pub member: Vec<String>,
    /// 수신자를 가리키는 필드 이름
    pub receiver_field: String,
    /// 메서드 이름을 가리키는 필드 이름
    pub method_field: String,
    /// 문자열 리터럴 노드 이름
    pub string: Vec<String>,
    /// 람다와 함수 리터럴 노드 이름. 인자에 있으면 핸들러 등록으로 본다.
    pub lambda: Vec<String>,
    /// 문자열을 이어 붙이는 식의 노드 이름. `"/api/x/" + id` 형태를 읽는다.
    #[serde(default)]
    pub concat: Vec<String>,
    /// `ApiPaths.ORDERS`처럼 한정된 이름의 노드. 상수를 별도 파일에 모아 두는
    /// 관례가 흔해서, 이 형태도 이름으로 읽어야 값을 찾을 수 있다.
    #[serde(default)]
    pub qualified_name: Vec<String>,
    /// 이름에 값을 묶는 선언 노드 이름. 상수 URL을 따라가는 데 쓴다.
    #[serde(default)]
    pub declaration: Vec<String>,
    /// 선언에서 이름을 가리키는 필드
    #[serde(default)]
    pub declaration_name_field: String,
    /// 선언에서 값을 가리키는 필드
    #[serde(default)]
    pub declaration_value_field: String,
    /// 실인자를 한 겹 더 감싸는 노드가 있으면 그 이름. C#의 `argument`가 그렇다.
    #[serde(default)]
    pub arg_wrapper: Option<String>,
    /// 호출식 자체가 수신자와 메서드 필드를 갖는가.
    /// Java의 `method_invocation`은 `function` 필드 없이 둘을 직접 갖는다.
    #[serde(default)]
    pub member_is_call: bool,
}

/// 메서드 선언에 붙은 어노테이션이 HTTP 라우트를 정의한다.
/// Spring `@GetMapping`, NestJS `@Get`, ASP.NET `[HttpGet]`, Micronaut `@Get` 등.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRule {
    pub lang: String,
    pub annotation: String,
    /// 이 어노테이션이 뜻하는 HTTP 메서드. `"ANY"`면 인자에서 찾아본다.
    pub method: String,
    /// `method = RequestMethod.POST` 처럼 인자가 메서드를 지정하는 형태를 허용한다.
    #[serde(default)]
    pub method_from_args_prefix: Option<String>,
    /// 데코레이터에 수신자가 붙는 형태(`@app.get`, `@router.post`)에서 허용할 수신자.
    ///
    /// 비어 있으면 수신자를 따지지 않는다(Java `@GetMapping`). 파이썬은
    /// `@cache.get` 같은 오탐을 막기 위해 이 목록이 필요하다.
    #[serde(default)]
    pub receivers: Vec<String>,
    /// `methods=["POST"]`(Flask)처럼 인자 안의 배열이 메서드를 지정하는 형태.
    #[serde(default)]
    pub method_from_args_list: Option<String>,
}

/// 클래스 어노테이션이 하위 라우트들의 경로 접두를 정한다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasePathRule {
    pub lang: String,
    pub annotation: String,
}

/// 클래스 어노테이션이 DI 컨테이너 등록 대상(스테레오타입)을 뜻한다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeanRule {
    pub lang: String,
    pub annotations: Vec<String>,
}

/// 주입 지점 판별 규칙.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectRule {
    pub lang: String,
    /// 필드에 붙으면 주입으로 보는 어노테이션
    #[serde(default)]
    pub annotations: Vec<String>,
    /// Lombok `@RequiredArgsConstructor` 관용구 — final 필드를 주입으로 본다
    #[serde(default)]
    pub final_fields: bool,
    /// 생성자 파라미터를 주입으로 본다
    #[serde(default)]
    pub constructor_params: bool,
}

/// HTTP 클라이언트 호출 — `CALLS_API` 엣지의 프런트 쪽 끝.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpClientRule {
    pub lang: String,
    /// 맨함수 호출. 예: `fetch("/api/x")`
    #[serde(default)]
    pub callee: Option<String>,
    /// 수신자 메서드 호출. 예: `axios.get(...)`, `apiClient.post(...)`
    /// 사내 래퍼(`http.fetchJson` 등)를 여기에 추가하면 그대로 지원된다.
    #[serde(default)]
    pub receiver_methods: Vec<String>,
    /// 고정 HTTP 메서드. 생략하면 호출된 메서드 이름을 대문자화해 쓴다.
    #[serde(default)]
    pub method: Option<String>,
    /// URL이 몇 번째 인자인지 (0-based)
    #[serde(default)]
    pub url_arg: usize,
    /// 이 수신자 이름이면 클라이언트 호출로 보지 않는다.
    ///
    /// `this.post(...)`(miragejs), `app.post(...)`(Express), `router.get(...)`은
    /// 라우트 **정의**이지 호출이 아니다. 실측에서 이 오탐이 21건 중 16건이었다.
    #[serde(default)]
    pub exclude_receivers: Vec<String>,
}

/// 영속 계층 판별. JPA · MyBatis · SQLAlchemy가 모두 이 틀에 들어간다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceRule {
    pub lang: String,
    /// 클래스를 엔티티로 표시하는 어노테이션 (`Entity`, `Table`)
    #[serde(default)]
    pub entity_annotations: Vec<String>,
    /// 테이블명을 담는 어노테이션 (`Table(name="orders")`)
    #[serde(default)]
    pub table_annotations: Vec<String>,
    /// 메서드에 붙어 SQL을 담는 어노테이션 (MyBatis `@Select`/`@Insert`)
    #[serde(default)]
    pub sql_annotations: Vec<String>,
    /// 이 이름의 클래스 속성이 테이블명을 담는다 (SQLAlchemy `__tablename__`)
    #[serde(default)]
    pub table_attribute: Option<String>,
    /// 이 상위 타입을 상속하면 리포지터리로 본다 (`JpaRepository`)
    #[serde(default)]
    pub repository_supertypes: Vec<String>,
}

/// 내장 기본 규칙. 정의는 `rules/builtin.*.toml`에 나뉘어 있다.
/// 설정이 비어 있어도 Spring + React가 바로 동작한다.
///
/// Rust 코드가 아니라 데이터 파일에 둔 이유가 있다. 규칙을 하나 더하는 일은
/// "이 어노테이션은 이 HTTP 메서드다"라는 사실을 적는 것뿐인데, 코드로 두면
/// `String` 변환과 `Vec` 생성 관용구를 알아야 한다. 프레임워크를 아는 사람이
/// Rust를 몰라서 기여하지 못할 이유가 없다.
///
/// 파일을 언어별로 나눈 것은 새 언어를 지원할 때 한 파일만 보게 하기
/// 위해서다. `include_str!`이 컴파일 시점에 내용을 넣으므로 배포물은 여전히
/// 실행 파일 하나다. tree-sitter 쿼리를 `queries/*.scm`에 둔 것과 같다.
///
/// # 패닉
///
/// 파일이 잘못되면 시작과 동시에 멈춘다. 잘못된 기본 규칙으로 조용히
/// 동작하는 것보다 낫고, `builtin_rules_parse` 테스트가 `cargo test`
/// 단계에서 먼저 잡는다.
pub fn builtin() -> FrameworkRules {
    /// 언어를 추가할 때 여기에 한 줄을 더한다. 빠뜨리면 그 언어의 규칙이
    /// 통째로 사라지므로 `builtin_covers_every_supported_language`가 잡는다.
    const FILES: &[(&str, &str)] = &[
        ("builtin.syntax.toml", include_str!("../rules/builtin.syntax.toml")),
        ("builtin.java.toml", include_str!("../rules/builtin.java.toml")),
        ("builtin.python.toml", include_str!("../rules/builtin.python.toml")),
        (
            "builtin.typescript.toml",
            include_str!("../rules/builtin.typescript.toml"),
        ),
        ("builtin.csharp.toml", include_str!("../rules/builtin.csharp.toml")),
    ];

    let mut merged = FrameworkRules::default();
    for (name, text) in FILES {
        let part: FrameworkRules = toml::from_str(text)
            .unwrap_or_else(|e| panic!("rules/{name} 파싱 실패 — 내장 규칙 파일이 깨졌다: {e}"));
        merged.absorb(part);
    }
    merged
}

impl FrameworkRules {
    /// 내장 기본값과 사용자 규칙을 합쳐 실제 적용될 규칙을 만든다.
    pub fn effective(user: &FrameworkRules) -> FrameworkRules {
        if user.replace_defaults {
            return user.clone();
        }
        let mut merged = builtin();
        merged.absorb(user.clone());
        merged
    }

    /// 다른 규칙 묶음을 뒤에 이어 붙인다. 앞에서부터 찾으므로 먼저 넣은
    /// 규칙이 우선한다.
    fn absorb(&mut self, other: FrameworkRules) {
        self.route.extend(other.route);
        self.base_path.extend(other.base_path);
        self.bean.extend(other.bean);
        self.inject.extend(other.inject);
        self.http_client.extend(other.http_client);
        self.persistence.extend(other.persistence);
        self.lang_syntax.extend(other.lang_syntax);
    }

    /// `lang`은 추출기 언어(`java`, `typescript`)와 맞춘다.
    /// javascript는 typescript 규칙을 함께 쓴다.
    fn lang_matches(rule_lang: &str, lang: &str) -> bool {
        rule_lang == lang
            || (rule_lang == "typescript" && lang == "javascript")
            || (rule_lang == "javascript" && lang == "typescript")
    }

    pub fn route_for(&self, lang: &str, annotation: &str) -> Option<&RouteRule> {
        self.route
            .iter()
            .find(|r| Self::lang_matches(&r.lang, lang) && r.annotation == annotation)
    }

    /// 수신자가 붙는 데코레이터(`app.get`)용. 수신자 허용 목록까지 확인한다.
    pub fn route_for_receiver(
        &self,
        lang: &str,
        receiver: &str,
        name: &str,
    ) -> Option<&RouteRule> {
        self.route.iter().find(|r| {
            Self::lang_matches(&r.lang, lang)
                && r.annotation == name
                && (r.receivers.is_empty()
                    || r.receivers.iter().any(|x| x.eq_ignore_ascii_case(receiver)))
        })
    }

    pub fn is_base_path_annotation(&self, lang: &str, annotation: &str) -> bool {
        self.base_path
            .iter()
            .any(|r| Self::lang_matches(&r.lang, lang) && r.annotation == annotation)
    }

    pub fn bean_stereotype(&self, lang: &str, annotation: &str) -> bool {
        self.bean.iter().any(|r| {
            Self::lang_matches(&r.lang, lang) && r.annotations.iter().any(|a| a == annotation)
        })
    }

    pub fn inject_rules(&self, lang: &str) -> Vec<&InjectRule> {
        self.inject
            .iter()
            .filter(|r| Self::lang_matches(&r.lang, lang))
            .collect()
    }

    pub fn persistence_rules(&self, lang: &str) -> Vec<&PersistenceRule> {
        self.persistence
            .iter()
            .filter(|r| Self::lang_matches(&r.lang, lang))
            .collect()
    }

    /// 그 언어의 구문 트리 노드 이름. 없으면 호출 탐지를 하지 않는다.
    pub fn lang_syntax(&self, lang: &str) -> Option<&LangSyntax> {
        self.lang_syntax.iter().find(|s| s.lang == lang)
    }

    pub fn http_clients(&self, lang: &str) -> Vec<&HttpClientRule> {
        self.http_client
            .iter()
            .filter(|r| Self::lang_matches(&r.lang, lang))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_covers_every_supported_language() {
        // 규칙 파일이 언어별로 나뉘어 있어서, `builtin()`의 FILES 목록에 한 줄을
        // 빠뜨리면 그 언어의 규칙이 통째로 사라진다. 컴파일은 그대로 되므로
        // 이 테스트가 대신 잡는다.
        let r = builtin();
        for lang in ["java", "python", "typescript", "csharp"] {
            assert!(
                r.lang_syntax(lang).is_some(),
                "{lang}의 구문 트리 정의가 없다 — builtin.syntax.toml 확인"
            );
        }
        // 각 언어 파일이 실제로 읽혔는지 대표 규칙 하나씩으로 확인한다.
        assert!(r.route_for("java", "GetMapping").is_some(), "builtin.java.toml");
        assert!(
            r.route_for_receiver("python", "app", "get").is_some(),
            "builtin.python.toml"
        );
        assert!(
            !r.http_clients("typescript").is_empty(),
            "builtin.typescript.toml"
        );
        assert!(r.route_for("csharp", "HttpGet").is_some(), "builtin.csharp.toml");
    }

    #[test]
    fn builtin_rules_parse() {
        // 기본값이 TOML 파일에 있으므로 필드 이름을 잘못 적어도 컴파일 오류가
        // 나지 않는다. 이 테스트가 그것을 다시 컴파일 단계의 오류로 만든다.
        let r = builtin();
        assert!(!r.route.is_empty(), "route 규칙을 읽지 못했다");
        assert!(!r.bean.is_empty(), "bean 규칙을 읽지 못했다");
        assert!(!r.inject.is_empty(), "inject 규칙을 읽지 못했다");
        assert!(!r.http_client.is_empty(), "http_client 규칙을 읽지 못했다");
        assert!(!r.persistence.is_empty(), "persistence 규칙을 읽지 못했다");
        assert!(!r.base_path.is_empty(), "base_path 규칙을 읽지 못했다");
        assert!(!r.lang_syntax.is_empty(), "lang_syntax를 읽지 못했다");
        assert!(
            !r.replace_defaults,
            "내장 규칙은 replace_defaults를 켜서는 안 된다"
        );
    }

    #[test]
    fn builtin_survives_a_serialization_round_trip() {
        // `nunchi rules --toml`이 이 규칙을 출력하고 사용자가 그것을 자기 설정에
        // 붙여 넣는다. 직렬화했다가 다시 읽어도 잃는 것이 없어야 한다.
        let original = builtin();
        let text = toml::to_string_pretty(&original).expect("직렬화 실패");
        let parsed: FrameworkRules = toml::from_str(&text).expect("역직렬화 실패");
        assert_eq!(original.route.len(), parsed.route.len());
        assert_eq!(original.http_client.len(), parsed.http_client.len());
        assert_eq!(original.persistence.len(), parsed.persistence.len());
    }

    #[test]
    fn builtin_covers_spring_and_react() {
        let r = FrameworkRules::effective(&FrameworkRules::default());
        assert_eq!(r.route_for("java", "GetMapping").unwrap().method, "GET");
        assert!(r.bean_stereotype("java", "Service"));
        assert!(r.is_base_path_annotation("java", "RequestMapping"));
        // javascript 파일도 typescript 규칙을 받는다
        assert!(!r.http_clients("javascript").is_empty());
    }

    #[test]
    fn builtin_excludes_route_definition_receivers() {
        let r = FrameworkRules::effective(&FrameworkRules::default());
        let receiver_rule = r
            .http_clients("typescript")
            .into_iter()
            .find(|c| !c.receiver_methods.is_empty())
            .unwrap();
        assert!(receiver_rule.exclude_receivers.iter().any(|x| x == "this"));
        assert!(receiver_rule.exclude_receivers.iter().any(|x| x == "app"));
    }

    #[test]
    fn builtin_covers_fastapi_flask_and_aspnet() {
        let r = FrameworkRules::effective(&FrameworkRules::default());
        // FastAPI: @app.get / @router.post
        assert!(r.route_for_receiver("python", "app", "get").is_some());
        assert!(r.route_for_receiver("python", "router", "post").is_some());
        // 임의 수신자는 라우트가 아니다 — @cache.get 오탐 방지
        assert!(r.route_for_receiver("python", "cache", "get").is_none());
        // Flask: @app.route(..., methods=["POST"])
        let flask = r.route_for_receiver("python", "app", "route").unwrap();
        assert_eq!(flask.method_from_args_list.as_deref(), Some("methods"));
        // ASP.NET
        assert_eq!(r.route_for("csharp", "HttpGet").unwrap().method, "GET");
        assert!(r.is_base_path_annotation("csharp", "Route"));
        assert!(r.bean_stereotype("csharp", "ApiController"));
    }

    #[test]
    fn builtin_covers_jpa_and_mybatis() {
        let r = FrameworkRules::effective(&FrameworkRules::default());
        let java = r.persistence_rules("java");
        assert!(!java.is_empty());
        let rule = java[0];
        assert!(rule.entity_annotations.iter().any(|a| a == "Entity"), "JPA");
        assert!(rule.sql_annotations.iter().any(|a| a == "Select"), "MyBatis");
        assert!(rule.repository_supertypes.iter().any(|a| a == "JpaRepository"));
        // SQLAlchemy
        assert_eq!(
            r.persistence_rules("python")[0].table_attribute.as_deref(),
            Some("__tablename__")
        );
    }

    #[test]
    fn user_rules_extend_defaults() {
        let user: FrameworkRules = toml::from_str(
            r#"
[[route]]
lang = "java"
annotation = "MyInternalEndpoint"
method = "POST"

[[http_client]]
lang = "typescript"
receiver_methods = ["fetchJson"]
method = "GET"
"#,
        )
        .unwrap();

        let r = FrameworkRules::effective(&user);
        // 사내 관용구가 추가되고
        assert_eq!(r.route_for("java", "MyInternalEndpoint").unwrap().method, "POST");
        assert!(r
            .http_clients("typescript")
            .iter()
            .any(|c| c.receiver_methods.iter().any(|m| m == "fetchJson")));
        // 기본값도 그대로 남는다
        assert!(r.route_for("java", "GetMapping").is_some());
    }

    #[test]
    fn replace_defaults_drops_builtins() {
        let user: FrameworkRules = toml::from_str(
            r#"
replace_defaults = true

[[route]]
lang = "java"
annotation = "Only"
method = "GET"
"#,
        )
        .unwrap();
        let r = FrameworkRules::effective(&user);
        assert!(r.route_for("java", "GetMapping").is_none());
        assert!(r.route_for("java", "Only").is_some());
    }
}

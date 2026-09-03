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
        }
    }
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

/// 내장 기본 규칙. 설정이 비어 있어도 Spring + React가 바로 동작한다.
pub fn builtin() -> FrameworkRules {
    let java_route = |anno: &str, method: &str| RouteRule {
        lang: "java".into(),
        annotation: anno.into(),
        method: method.into(),
        method_from_args_prefix: None,
        receivers: Vec::new(),
        method_from_args_list: None,
    };
    // FastAPI `@app.get("/x")` · Flask `@app.route("/x", methods=["POST"])`
    let py_receivers: Vec<String> = ["app", "router", "api", "blueprint", "bp"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let py_route = |name: &str, method: &str| RouteRule {
        lang: "python".into(),
        annotation: name.into(),
        method: method.into(),
        method_from_args_prefix: None,
        receivers: py_receivers.clone(),
        method_from_args_list: None,
    };
    // ASP.NET `[HttpGet("orders")]`
    let cs_route = |anno: &str, method: &str| RouteRule {
        lang: "csharp".into(),
        annotation: anno.into(),
        method: method.into(),
        method_from_args_prefix: None,
        receivers: Vec::new(),
        method_from_args_list: None,
    };

    FrameworkRules {
        replace_defaults: false,
        route: vec![
            java_route("GetMapping", "GET"),
            java_route("PostMapping", "POST"),
            java_route("PutMapping", "PUT"),
            java_route("DeleteMapping", "DELETE"),
            java_route("PatchMapping", "PATCH"),
            RouteRule {
                lang: "java".into(),
                annotation: "RequestMapping".into(),
                method: "ANY".into(),
                method_from_args_prefix: Some("RequestMethod.".into()),
                receivers: Vec::new(),
                method_from_args_list: None,
            },
            py_route("get", "GET"),
            py_route("post", "POST"),
            py_route("put", "PUT"),
            py_route("delete", "DELETE"),
            py_route("patch", "PATCH"),
            RouteRule {
                lang: "python".into(),
                annotation: "route".into(),
                method: "ANY".into(),
                method_from_args_prefix: None,
                receivers: py_receivers.clone(),
                // Flask는 methods=["POST"] 로 지정한다
                method_from_args_list: Some("methods".into()),
            },
            cs_route("HttpGet", "GET"),
            cs_route("HttpPost", "POST"),
            cs_route("HttpPut", "PUT"),
            cs_route("HttpDelete", "DELETE"),
            cs_route("HttpPatch", "PATCH"),
        ],
        base_path: vec![
            BasePathRule { lang: "java".into(), annotation: "RequestMapping".into() },
            // ASP.NET: [Route("api/[controller]")]
            BasePathRule { lang: "csharp".into(), annotation: "Route".into() },
        ],
        bean: vec![
            BeanRule {
            lang: "csharp".into(),
            annotations: ["ApiController", "Controller"].iter().map(|s| s.to_string()).collect(),
        },
        BeanRule {
            lang: "java".into(),
            annotations: [
                "RestController",
                "Controller",
                "Service",
                "Repository",
                "Component",
                "Configuration",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        }],
        inject: vec![InjectRule {
            lang: "java".into(),
            annotations: vec!["Autowired".into(), "Inject".into()],
            final_fields: true,
            constructor_params: true,
        }],
        persistence: vec![
            PersistenceRule {
                lang: "java".into(),
                entity_annotations: vec!["Entity".into()],
                table_annotations: vec!["Table".into()],
                // MyBatis 어노테이션 매퍼 — 한국 기업 Spring 환경에서 비중이 크다.
                sql_annotations: ["Select", "Insert", "Update", "Delete", "SelectProvider"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                table_attribute: None,
                repository_supertypes: ["JpaRepository", "CrudRepository", "MongoRepository"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            },
            PersistenceRule {
                lang: "python".into(),
                entity_annotations: Vec::new(),
                table_annotations: Vec::new(),
                sql_annotations: Vec::new(),
                // SQLAlchemy
                table_attribute: Some("__tablename__".into()),
                repository_supertypes: Vec::new(),
            },
        ],
        http_client: vec![
            HttpClientRule {
                lang: "typescript".into(),
                callee: Some("fetch".into()),
                receiver_methods: Vec::new(),
                method: Some("GET".into()),
                url_arg: 0,
                exclude_receivers: Vec::new(),
            },
            HttpClientRule {
                lang: "typescript".into(),
                callee: None,
                receiver_methods: ["get", "post", "put", "delete", "patch", "head", "options"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                // method 생략 → 호출된 메서드 이름이 곧 HTTP 메서드
                method: None,
                url_arg: 0,
                exclude_receivers: ["this", "app", "router", "server", "mock"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            },
            // requests / httpx / aiohttp — 파이썬 HTTP 클라이언트
            HttpClientRule {
                lang: "python".into(),
                callee: None,
                receiver_methods: ["get", "post", "put", "delete", "patch"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                method: None,
                url_arg: 0,
                // app·router는 라우트 정의다. cache·session은 HTTP가 아닐 수 있으나
                // session.get(url)은 실제 클라이언트이므로 제외하지 않는다.
                exclude_receivers: ["app", "router", "blueprint", "bp", "cache", "redis"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            },
            // C# HttpClient
            HttpClientRule {
                lang: "csharp".into(),
                callee: None,
                receiver_methods: ["GetAsync", "PostAsync", "PutAsync", "DeleteAsync", "PatchAsync"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                method: None,
                url_arg: 0,
                exclude_receivers: Vec::new(),
            },
        ],
    }
}

impl FrameworkRules {
    /// 내장 기본값과 사용자 규칙을 합쳐 실제 적용될 규칙을 만든다.
    pub fn effective(user: &FrameworkRules) -> FrameworkRules {
        if user.replace_defaults {
            return user.clone();
        }
        let mut merged = builtin();
        merged.route.extend(user.route.iter().cloned());
        merged.base_path.extend(user.base_path.iter().cloned());
        merged.bean.extend(user.bean.iter().cloned());
        merged.inject.extend(user.inject.iter().cloned());
        merged.http_client.extend(user.http_client.iter().cloned());
        merged.persistence.extend(user.persistence.iter().cloned());
        merged
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

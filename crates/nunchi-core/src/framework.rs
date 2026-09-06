//! 프레임워크 의미론 추출 — Spring / React (docs/DESIGN.md 4·5절 Phase 1c)
//!
//! 이 계층이 없으면 그래프가 무너진다. Spring은 호출 관계가 **어노테이션과 DI로
//! 구성**되어 소스에 구문적 호출이 존재하지 않기 때문이다. 실측에서 확인된 바:
//! RealWorld 인덱싱 시 미해소 호출 상위가 `save`(JPA 리포지터리 — 본문 없음),
//! `build`/`builder`(Lombok 생성 코드)였다.
//!
//! 어노테이션-선언 결합은 tree-sitter 쿼리로 표현하면 취약해서 수동 순회로 처리한다.

use crate::model::Span;
use crate::rules::FrameworkRules;
use tree_sitter::Node;

#[derive(Debug, Default, Clone)]
pub struct FrameworkFacts {
    pub routes: Vec<RouteFact>,
    pub beans: Vec<BeanFact>,
    pub injects: Vec<InjectFact>,
    pub api_calls: Vec<ApiCallFact>,
    /// 이 파일에 선언된 이름과 문자열 값. 다른 파일이 참조할 수 있으므로
    /// 인덱싱 2패스에서 합쳐 미해결 URL을 다시 해소한다.
    pub string_constants: std::collections::HashMap<String, String>,
    pub entities: Vec<EntityFact>,
    /// (소유 심볼, 테이블명) — SQL 어노테이션·매퍼에서 추출
    pub table_refs: Vec<TableRefFact>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityFact {
    /// 클래스 이름
    pub name: String,
    /// 매핑되는 테이블. 명시가 없으면 클래스명에서 추정하지 않고 None으로 둔다.
    pub table: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableRefFact {
    /// 이 SQL을 담은 심볼(메서드) 이름
    pub owner: String,
    pub table: String,
    /// select / insert / update / delete
    pub verb: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteFact {
    pub method: String,
    /// 정규화된 경로 — `/api/orders/{}` 형태
    pub path: String,
    /// 원본 표기 (표시용)
    pub raw_path: String,
    /// 이 라우트를 처리하는 심볼 이름
    pub handler: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeanFact {
    pub name: String,
    /// `RestController`, `Service`, `Repository` 등
    pub stereotype: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InjectFact {
    /// 주입받는 쪽 (Bean 이름)
    pub owner: String,
    /// 주입되는 타입 이름
    pub injected_type: String,
    pub how: InjectKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectKind {
    Autowired,
    Constructor,
    /// Lombok `@RequiredArgsConstructor` + final 필드
    FinalField,
}

// ANCHOR: url_part
/// 경로를 이루는 조각 하나.
///
/// 코드에 적힌 URL은 리터럴만인 경우가 드물다. 상수를 이어 붙이거나 변수를
/// 끼워 넣는다. 그 구조를 문자열에 기호로 심어 두면 단계마다 다시 파싱해야
/// 하므로, 조각인 채로 들고 다니다가 마지막에 한 번만 문자열로 만든다.
#[derive(Debug, Clone, PartialEq)]
pub enum UrlPart {
    /// 코드에 그대로 적힌 부분
    Literal(String),
    /// 값을 알 수 없는 자리. 변수나 함수 호출이다.
    Unknown,
    /// 이름은 알지만 값은 아직 모른다. 경로 상수를 별도 파일에 모아 두는
    /// 관례가 흔해서, 인덱싱 2패스가 전체 파일의 상수 표로 이 자리를 채운다.
    Named(String),
}
// ANCHOR_END: url_part

/// 조각으로 표현한 경로.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UrlTemplate(Vec<UrlPart>);

impl UrlTemplate {
    // ANCHOR: url_template
    /// 알 수 있는 것이 하나도 없으면 URL로 볼 수 없다.
    /// `getForObject(props.getUrl(), ...)`처럼 값을 전혀 모르는 호출이다.
    pub fn is_meaningful(&self) -> bool {
        self.0.iter().any(|p| !matches!(p, UrlPart::Unknown))
    }

    /// 아직 값을 못 찾은 이름이 있는가. 있으면 2패스에서 채운다.
    pub fn has_unresolved(&self) -> bool {
        self.0.iter().any(|p| matches!(p, UrlPart::Named(_)))
    }

    /// 이름 자리를 상수 표의 값으로 채운다.
    /// 표에 없으면 끝내 알 수 없는 값이므로 `Unknown`이 된다.
    pub fn fill(&mut self, table: &std::collections::HashMap<String, String>) {
        for part in self.0.iter_mut() {
            if let UrlPart::Named(name) = part {
                *part = match table.get(name.as_str()) {
                    Some(value) if !value.is_empty() => UrlPart::Literal(value.clone()),
                    _ => UrlPart::Unknown,
                };
            }
        }
    }

    /// 경로를 확정할 수 없는가.
    ///
    /// 값을 모르는 자리가 세그먼트 하나를 통째로 차지하면 경로 파라미터로 본다.
    /// Spring의 `/{id}`가 정규화된 결과와 같은 문자열이 되므로 라우트에 연결된다.
    /// 세그먼트 중간에 끼어 있으면(`"/api/orders" + suffix`) 어떤 엔드포인트가
    /// 되는지 알 수 없으므로 연결 대상에서 뺀다.
    pub fn is_dynamic(&self) -> bool {
        self.0.iter().enumerate().any(|(i, part)| {
            if !matches!(part, UrlPart::Unknown) {
                return false;
            }
            let left_ends = match i.checked_sub(1).and_then(|j| self.0.get(j)) {
                Some(UrlPart::Literal(s)) => s.ends_with('/'),
                _ => false,
            };
            let right_starts = match self.0.get(i + 1) {
                Some(UrlPart::Literal(s)) => s.starts_with('/'),
                None => true,
                _ => false,
            };
            !(left_ends && right_starts)
        })
    }
    // ANCHOR_END: url_template

    // ANCHOR: render
    /// 최종 경로 문자열. 값을 모르는 자리는 `{}`가 된다.
    /// 이 프로젝트에서 `{}` 표기를 만들어 내는 유일한 자리다.
    pub fn render(&self) -> String {
        self.0
            .iter()
            .map(|p| match p {
                UrlPart::Literal(s) => s.as_str(),
                _ => "{}",
            })
            .collect()
    }
    // ANCHOR_END: render

    /// 사람이 읽을 원문. 아직 못 채운 이름을 그대로 보여 준다.
    pub fn describe(&self) -> String {
        self.0
            .iter()
            .map(|p| match p {
                UrlPart::Literal(s) => s.clone(),
                UrlPart::Named(n) => format!("<{n}>"),
                UrlPart::Unknown => "{}".to_string(),
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiCallFact {
    pub method: String,
    pub path: String,
    pub raw_path: String,
    /// 경로 조각. 아직 못 채운 이름이 있으면 2패스가 이것으로 다시 만든다.
    pub template: UrlTemplate,
    pub span: Span,
    /// 경로가 정적으로 결정되지 않는다 — 예: `` `/users${isRegister ? '' : '/login'}` ``.
    /// 치환이 경로 세그먼트 전체가 아니면 어떤 엔드포인트인지 알 수 없다.
    /// 연결 실패로 세지 않고 별도로 보고한다.
    pub dynamic: bool,
}

/// 경로 템플릿 정규화.
///
/// Spring `{id}`, Express/react-router `:id`, JS 템플릿 `${id}`를 모두 `{}`로 만들어
/// 프런트–백엔드 매칭이 문자열 비교로 끝나게 한다 (docs/DESIGN.md 4·5절 `CALLS_API`).
/// 경로 치환이 세그먼트 전체가 아닌지 판정한다.
///
/// `/orders/${id}` 는 경로 파라미터라 `{}` 로 정규화하면 되지만,
/// `/users${cond ? '' : '/login'}` 은 세그먼트 경계가 아니어서 정적으로 알 수 없다.
pub fn has_dynamic_segment(raw: &str) -> bool {
    let trimmed = raw.trim().trim_matches(['"', '\'', '`']);
    let bytes = trimmed.as_bytes();
    let mut i = 0;
    while let Some(pos) = trimmed[i..].find("${") {
        let start = i + pos;
        // 치환 앞이 `/` 가 아니면 세그먼트 중간이다.
        if start == 0 || bytes[start - 1] != b'/' {
            return true;
        }
        let Some(end_rel) = trimmed[start..].find('}') else { return true };
        let end = start + end_rel + 1;
        // 치환 뒤가 `/` 또는 끝이 아니면 세그먼트 중간이다.
        if end < bytes.len() && bytes[end] != b'/' {
            return true;
        }
        i = end;
    }
    false
}

pub fn normalize_route_path(raw: &str) -> String {
    let trimmed = raw.trim().trim_matches(['"', '\'', '`']);
    let mut out = String::with_capacity(trimmed.len());
    let mut chars = trimmed.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // `${...}` (JS 템플릿) 과 `{...}` (Spring)
            '$' if chars.peek() == Some(&'{') => {
                chars.next();
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                }
                out.push_str("{}");
            }
            '{' => {
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                }
                out.push_str("{}");
            }
            // `:id` (Express / react-router) — 세그먼트 전체가 파라미터
            ':' if out.ends_with('/') => {
                while chars.peek().is_some_and(|c| *c != '/') {
                    chars.next();
                }
                out.push_str("{}");
            }
            _ => out.push(c),
        }
    }

    // 선행 슬래시 보장 + 후행 슬래시 제거 (`/api/orders/` ≡ `/api/orders`)
    let mut path = if out.starts_with('/') { out } else { format!("/{out}") };
    while path.len() > 1 && path.ends_with('/') {
        path.pop();
    }
    path
}

fn span_of(node: Node) -> Span {
    Span {
        start_line: node.start_position().row as u32 + 1,
        end_line: node.end_position().row as u32 + 1,
    }
}

fn text<'a>(node: Node, src: &'a [u8]) -> &'a str {
    node.utf8_text(src).unwrap_or_default()
}

// ─────────────────────────── Java / Spring ───────────────────────────

/// 선언 노드에 붙은 어노테이션/어트리뷰트를 `(이름, 인자원문)`으로 돌려준다.
///
/// 두 형태를 모두 다룬다:
/// - Java: `modifiers > marker_annotation | annotation`
/// - C#:   `attribute_list > attribute > identifier + attribute_argument_list`
fn annotations_of<'a>(decl: Node<'a>, src: &'a [u8]) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    let mut cursor = decl.walk();
    for child in decl.children(&mut cursor) {
        // C# 어트리뷰트
        if child.kind() == "attribute_list" {
            let mut attrs = child.walk();
            for attr in child.children(&mut attrs) {
                if attr.kind() != "attribute" {
                    continue;
                }
                let mut parts = attr.walk();
                let mut name = None;
                let mut args = None;
                for piece in attr.children(&mut parts) {
                    match piece.kind() {
                        "identifier" | "qualified_name" if name.is_none() => {
                            name = Some(text(piece, src).to_string());
                        }
                        "attribute_argument_list" => {
                            args = Some(text(piece, src).to_string());
                        }
                        _ => {}
                    }
                }
                if let Some(name) = name {
                    out.push((name, args));
                }
            }
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
                    let args = m
                        .child_by_field_name("arguments")
                        .map(|a| text(a, src).to_string());
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

/// `("/api/orders")` 또는 `(value = "/x", method = RequestMethod.GET)` 에서 경로를 뽑는다.
fn path_from_args(args: &str) -> Option<String> {
    // 명명 인자가 있으면 value/path 를 우선한다.
    for key in ["value = ", "path = ", "value=", "path="] {
        if let Some(idx) = args.find(key) {
            let rest = &args[idx + key.len()..];
            if let Some(s) = first_string_literal(rest) {
                return Some(s);
            }
        }
    }
    first_string_literal(args)
}

fn first_string_literal(s: &str) -> Option<String> {
    first_string_literal_any(s)
}

/// `method = RequestMethod.POST` → `POST` (접두는 규칙이 정한다)
fn method_from_args(args: &str, prefix: &str) -> Option<String> {
    let idx = args.find(prefix)?;
    let rest = &args[idx + prefix.len()..];
    let verb: String = rest.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
    (!verb.is_empty()).then_some(verb)
}

/// 어노테이션 기반 프레임워크 추출. 규칙은 설정에서 온다(`crate::rules`).
pub fn extract_annotated(
    root: Node,
    src: &[u8],
    lang: &str,
    rules: &FrameworkRules,
) -> FrameworkFacts {
    let mut facts = FrameworkFacts::default();
    match lang {
        // 파이썬은 데코레이터가 어노테이션 자리를 대신한다.
        "python" => walk_python(root, src, lang, rules, &mut facts),
        // C# 어트리뷰트 `[HttpGet("x")]` 는 Java 어노테이션과 구조가 같다.
        _ => walk_java(root, src, lang, rules, "", &mut facts),
    }
    facts
}

// ─────────────────────────── Python (FastAPI / Flask) ───────────────────────────

/// `@app.get("/orders")` 를 `(수신자, 이름, 인자원문)` 으로 분해한다.
fn python_decorators<'a>(node: Node<'a>, src: &'a [u8]) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    let Some(parent) = node.parent() else { return out };
    if parent.kind() != "decorated_definition" {
        return out;
    }
    let mut cursor = parent.walk();
    for child in parent.children(&mut cursor) {
        if child.kind() != "decorator" {
            continue;
        }
        let raw = text(child, src).trim_start_matches('@');
        // `app.get("/x")` → 수신자 `app`, 이름 `get`, 인자 `("/x")`
        let (head, args) = match raw.find('(') {
            Some(i) => (&raw[..i], raw[i..].to_string()),
            None => (raw, String::new()),
        };
        let (receiver, name) = match head.rsplit_once('.') {
            Some((r, n)) => (r.to_string(), n.to_string()),
            None => (String::new(), head.to_string()),
        };
        out.push((receiver, name.trim().to_string(), args));
    }
    out
}

fn walk_python(
    node: Node,
    src: &[u8],
    lang: &str,
    rules: &FrameworkRules,
    facts: &mut FrameworkFacts,
) {
    if node.kind() == "function_definition" {
        let handler = node
            .child_by_field_name("name")
            .map(|n| text(n, src).to_string())
            .unwrap_or_default();
        for (receiver, name, args) in python_decorators(node, src) {
            let Some(rule) = rules.route_for_receiver(lang, &receiver, &name) else { continue };
            // Flask: methods=["POST"]
            let method = rule
                .method_from_args_list
                .as_deref()
                .and_then(|key| method_from_args_list(&args, key))
                .unwrap_or_else(|| rule.method.clone());
            let raw = first_string_literal(&args).unwrap_or_default();
            let path = normalize_route_path(&raw);
            facts.routes.push(RouteFact {
                method,
                path: if path.is_empty() { "/".into() } else { path },
                raw_path: if raw.is_empty() { "/".into() } else { raw },
                handler: handler.clone(),
                span: span_of(node),
            });
        }
    }

    // SQLAlchemy: class Order(Base): __tablename__ = "orders"
    if node.kind() == "class_definition" {
        let name = node
            .child_by_field_name("name")
            .map(|n| text(n, src).to_string())
            .unwrap_or_default();
        for rule in rules.persistence_rules(lang) {
            let Some(attr) = rule.table_attribute.as_deref() else { continue };
            let body = text(node, src);
            if let Some(pos) = body.find(attr) {
                if let Some(table) = first_string_literal(&body[pos..]) {
                    facts.entities.push(EntityFact {
                        name: name.clone(),
                        table: Some(table),
                        span: span_of(node),
                    });
                    break;
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_python(child, src, lang, rules, facts);
    }
}

/// `methods=["POST", "GET"]` 에서 첫 메서드를 꺼낸다.
fn method_from_args_list(args: &str, key: &str) -> Option<String> {
    let pos = args.find(key)?;
    let rest = &args[pos + key.len()..];
    let open = rest.find('[')?;
    let close = rest[open..].find(']')? + open;
    first_string_literal_any(&rest[open..close]).map(|m| m.to_uppercase())
}

fn first_string_literal_any(s: &str) -> Option<String> {
    for quote in ['"', '\''] {
        if let Some(start) = s.find(quote) {
            let rest = &s[start + 1..];
            if let Some(end) = rest.find(quote) {
                return Some(rest[..end].to_string());
            }
        }
    }
    None
}

fn walk_java(
    node: Node,
    src: &[u8],
    lang: &str,
    rules: &FrameworkRules,
    base_path: &str,
    facts: &mut FrameworkFacts,
) {
    if node.kind() == "class_declaration" {
        let name = node
            .child_by_field_name("name")
            .map(|n| text(n, src).to_string())
            .unwrap_or_default();
        let annos = annotations_of(node, src);

        let stereotype = annos
            .iter()
            .find(|(a, _)| rules.bean_stereotype(lang, a))
            .map(|(a, _)| a.clone());

        // 클래스 레벨 @RequestMapping은 메서드 경로의 접두가 된다.
        let class_base = annos
            .iter()
            .find(|(a, _)| rules.is_base_path_annotation(lang, a))
            .and_then(|(_, args)| args.as_deref())
            .and_then(path_from_args)
            .map(|p| normalize_route_path(&p))
            .unwrap_or_default();
        let class_base = if class_base == "/" { String::new() } else { class_base };

        // 엔티티 — @Entity / @Table(name="orders")
        for rule in rules.persistence_rules(lang) {
            let is_entity = annos
                .iter()
                .any(|(a, _)| rule.entity_annotations.iter().any(|w| w == a));
            let table = annos
                .iter()
                .find(|(a, _)| rule.table_annotations.iter().any(|w| w == a))
                .and_then(|(_, args)| args.as_deref())
                .and_then(path_from_args);
            if is_entity || table.is_some() {
                facts.entities.push(EntityFact {
                    name: name.clone(),
                    table,
                    span: span_of(node),
                });
                break;
            }
        }

        if let Some(stereotype) = &stereotype {
            facts.beans.push(BeanFact {
                name: name.clone(),
                stereotype: stereotype.clone(),
                span: span_of(node),
            });
            collect_java_injections(node, src, lang, rules, &name, facts);
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            walk_java(child, src, lang, rules, &class_base, facts);
        }
        return;
    }

    if node.kind() == "method_declaration" {
        // MyBatis 어노테이션 매퍼 — @Select("SELECT ... FROM orders")
        let method_name = node
            .child_by_field_name("name")
            .map(|n| text(n, src).to_string())
            .unwrap_or_default();
        for (anno, args) in annotations_of(node, src) {
            let is_sql = rules
                .persistence_rules(lang)
                .iter()
                .any(|r| r.sql_annotations.iter().any(|w| *w == anno));
            if !is_sql {
                continue;
            }
            let Some(args) = args else { continue };
            for (table, verb) in tables_in_sql(&args) {
                facts.table_refs.push(TableRefFact {
                    owner: method_name.clone(),
                    table,
                    verb,
                    span: span_of(node),
                });
            }
        }

        let handler = node
            .child_by_field_name("name")
            .map(|n| text(n, src).to_string())
            .unwrap_or_default();
        for (anno, args) in annotations_of(node, src) {
            let Some(rule) = rules.route_for(lang, &anno) else { continue };
            let args_text = args.unwrap_or_default();
            let method = rule
                .method_from_args_prefix
                .as_deref()
                .and_then(|prefix| method_from_args(&args_text, prefix))
                .unwrap_or_else(|| rule.method.clone());
            let raw = path_from_args(&args_text).unwrap_or_default();
            let suffix = normalize_route_path(&raw);
            let suffix = if suffix == "/" { String::new() } else { suffix };
            let full = format!("{base_path}{suffix}");
            let full = if full.is_empty() { "/".to_string() } else { full };

            facts.routes.push(RouteFact {
                method,
                path: full,
                raw_path: if raw.is_empty() { "/".into() } else { raw },
                handler: handler.clone(),
                span: span_of(node),
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_java(child, src, lang, rules, base_path, facts);
    }
}

/// Bean 클래스의 주입 지점. Spring은 세 가지 형태를 모두 쓴다.
fn collect_java_injections(
    class_node: Node,
    src: &[u8],
    lang: &str,
    rules: &FrameworkRules,
    owner: &str,
    facts: &mut FrameworkFacts,
) {
    let Some(body) = class_node.child_by_field_name("body") else { return };
    let inject_rules = rules.inject_rules(lang);
    if inject_rules.is_empty() {
        return;
    }
    let mut cursor = body.walk();

    for member in body.children(&mut cursor) {
        match member.kind() {
            "field_declaration" => {
                let Some(ty) = member.child_by_field_name("type") else { continue };
                let type_name = text(ty, src).to_string();
                let annos = annotations_of(member, src);
                let autowired = inject_rules.iter().any(|r| {
                    r.annotations
                        .iter()
                        .any(|want| annos.iter().any(|(a, _)| a == want))
                });
                // Lombok @RequiredArgsConstructor 관용구 — final 필드가 곧 생성자 주입이다.
                let is_final = inject_rules.iter().any(|r| r.final_fields)
                    && text(member, src).contains("final ");

                if autowired || is_final {
                    facts.injects.push(InjectFact {
                        owner: owner.to_string(),
                        injected_type: type_name,
                        how: if autowired { InjectKind::Autowired } else { InjectKind::FinalField },
                    });
                }
            }
            "constructor_declaration" => {
                if !inject_rules.iter().any(|r| r.constructor_params) {
                    continue;
                }
                let Some(params) = member.child_by_field_name("parameters") else { continue };
                let mut pc = params.walk();
                for p in params.children(&mut pc) {
                    if p.kind() != "formal_parameter" {
                        continue;
                    }
                    if let Some(ty) = p.child_by_field_name("type") {
                        facts.injects.push(InjectFact {
                            owner: owner.to_string(),
                            injected_type: text(ty, src).to_string(),
                            how: InjectKind::Constructor,
                        });
                    }
                }
            }
            _ => {}
        }
    }
}

/// SQL 문자열에서 테이블 이름을 뽑는다.
///
/// 파서를 붙이지 않는다 — MyBatis XML과 어노테이션에 들어가는 SQL은 `#{param}`,
/// `<if>` 같은 템플릿 조각이 섞여 있어 정식 파싱이 자주 실패한다.
/// FROM/JOIN/INTO/UPDATE 뒤의 식별자만 집는 편이 견고하다.
pub fn tables_in_sql(sql: &str) -> Vec<(String, String)> {
    const KEYWORDS: &[(&str, &str)] = &[
        ("from", "select"),
        ("join", "select"),
        ("into", "insert"),
        ("update", "update"),
        ("delete from", "delete"),
    ];
    let lower = sql.to_lowercase();
    let mut out: Vec<(String, String)> = Vec::new();

    for (kw, verb) in KEYWORDS {
        let mut from = 0usize;
        while let Some(pos) = lower[from..].find(kw) {
            let start = from + pos;
            let after = start + kw.len();
            from = after;
            // 단어 경계 확인 — `fromage` 같은 오탐 방지
            let before_ok = start == 0 || !lower.as_bytes()[start - 1].is_ascii_alphanumeric();
            if !before_ok || after >= lower.len() {
                continue;
            }
            let rest = &sql[after..];
            let ident: String = rest
                .trim_start()
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
                .collect();
            // 스키마 접두(`dbo.orders`)는 마지막 조각만 쓴다.
            let table = ident.rsplit('.').next().unwrap_or("").to_string();
            if table.len() < 2 || table.chars().next().is_some_and(|c| c.is_numeric()) {
                continue;
            }
            // SQL 예약어가 잡히는 경우 제외
            const RESERVED: &[&str] = &["select", "where", "set", "values", "and", "or", "on"];
            if RESERVED.contains(&table.to_lowercase().as_str()) {
                continue;
            }
            let entry = (table, verb.to_string());
            if !out.contains(&entry) {
                out.push(entry);
            }
        }
    }
    out
}

// ─────────────────────────── HTTP 클라이언트 호출 ───────────────────────────

/// `fetch("/api/x")`, `axios.get("/api/x")`, `restTemplate.getForObject("/api/x", ...)`를
/// 잡는다. 이것이 `CALLS_API` 엣지의 부르는 쪽 끝이다.
///
/// 프런트엔드만 HTTP를 부르는 것이 아니다. 백엔드도 다른 서비스나 외부 API를
/// 부르므로 네 언어를 모두 지원한다.
pub fn extract_api_calls(
    root: Node,
    src: &[u8],
    lang: &str,
    rules: &FrameworkRules,
) -> Vec<ApiCallFact> {
    let clients = rules.http_clients(lang);
    let mut out = Vec::new();
    if clients.is_empty() {
        return out;
    }
    let Some(syntax) = rules.lang_syntax(lang) else {
        return out;
    };
    // 파일을 한 번 훑어 이름에 묶인 문자열을 모아 둔다. 호출 자리에 이름만
    // 있을 때 이 표에서 값을 찾는다.
    let constants = string_constants(root, src, syntax);
    walk_calls(root, src, &clients, syntax, &constants, &mut out);
    out
}

fn walk_calls(
    node: Node,
    src: &[u8],
    clients: &[&crate::rules::HttpClientRule],
    syntax: &crate::rules::LangSyntax,
    constants: &std::collections::HashMap<String, String>,
    out: &mut Vec<ApiCallFact>,
) {
    if syntax.call.iter().any(|k| k == node.kind()) {
        if let Some(fact) = api_call_of(node, src, clients, syntax, constants) {
            out.push(fact);
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_calls(child, src, clients, syntax, constants, out);
    }
}

/// 호출 대상을 수신자와 메서드 이름으로 나눈다.
/// `fetch(...)`처럼 수신자가 없으면 첫 값이 `None`이다.
fn callee_of(call: Node, src: &[u8], syntax: &crate::rules::LangSyntax) -> Option<(Option<String>, String)> {
    let member = if syntax.member_is_call {
        call
    } else {
        let func = call.child_by_field_name("function")?;
        if !syntax.member.iter().any(|k| k == func.kind()) {
            return Some((None, text(func, src).to_string()));
        }
        func
    };
    let name = member.child_by_field_name(&syntax.method_field)?;
    let receiver = member
        .child_by_field_name(&syntax.receiver_field)
        .map(|r| text(r, src).to_string());
    Some((receiver, text(name, src).to_string()))
}

fn api_call_of(
    call: Node,
    src: &[u8],
    clients: &[&crate::rules::HttpClientRule],
    syntax: &crate::rules::LangSyntax,
    constants: &std::collections::HashMap<String, String>,
) -> Option<ApiCallFact> {
    let (receiver, callee) = callee_of(call, src, syntax)?;

    // 어떤 규칙에 걸리는지 찾는다. 사내 래퍼도 설정에 추가하면 그대로 잡힌다.
    let (method, url_arg) = clients.iter().find_map(|rule| -> Option<(String, usize)> {
        match receiver.as_deref() {
            // `fetch("/api/x")`처럼 수신자 없이 부르는 형태
            None => (rule.callee.as_deref() == Some(callee.as_str()))
                .then(|| (rule.method.clone().unwrap_or_else(|| "GET".into()), rule.url_arg)),
            Some(recv) => {
                // `this.post(...)`, `app.get(...)`은 라우트 정의다.
                if rule
                    .exclude_receivers
                    .iter()
                    .any(|x| x.eq_ignore_ascii_case(recv))
                {
                    return None;
                }
                let verb = callee.to_ascii_lowercase();
                rule.receiver_methods
                    .iter()
                    .any(|m| m.eq_ignore_ascii_case(&verb))
                    .then(|| {
                        // method 미지정이면 호출된 메서드 이름이 곧 HTTP 메서드다.
                        // C# `GetAsync`처럼 접미가 붙는 관용구를 벗긴다.
                        let m = rule
                            .method
                            .clone()
                            .unwrap_or_else(|| verb.trim_end_matches("async").to_uppercase());
                        (m, rule.url_arg)
                    })
            }
        }
    })?;

    let args = call.child_by_field_name("arguments")?;

    // 인자에 함수나 람다가 있으면 **핸들러 등록**이다. 클라이언트 호출이 아니다.
    // Express `app.get(path, handler)`, miragejs `this.post(path, handler)` 등을
    // 프레임워크와 무관하게 걸러낸다.
    if has_function_argument(args, syntax) {
        return None;
    }

    let template = url_argument_at(args, src, url_arg, syntax, constants)?;
    // 알 수 있는 것이 하나도 없으면 어떤 경로인지 정할 수 없다.
    if !template.is_meaningful() {
        return None;
    }
    // 도메인 경로처럼 보이지 않으면 버린다. `useState("")` 같은 오탐 방지.
    //
    // 다만 아직 값을 못 찾은 이름이 남아 있으면 여기서 판단할 수 없다.
    // 다른 파일에 선언된 상수일 수 있으므로 2패스로 미룬다.
    let rendered = template.render();
    if !template.has_unresolved() && !rendered.starts_with('/') && !rendered.contains("/api") {
        return None;
    }

    Some(ApiCallFact {
        method,
        path: rendered,
        raw_path: template.describe(),
        dynamic: template.is_dynamic(),
        template,
        span: span_of(call),
    })
}

/// C#처럼 실인자가 한 겹 감싸여 있으면 안쪽 노드를 꺼낸다.
fn unwrap_argument<'a>(arg: Node<'a>, syntax: &crate::rules::LangSyntax) -> Node<'a> {
    match syntax.arg_wrapper.as_deref() {
        Some(wrapper) if arg.kind() == wrapper => arg.named_child(0).unwrap_or(arg),
        _ => arg,
    }
}

fn has_function_argument(args: Node, syntax: &crate::rules::LangSyntax) -> bool {
    let mut cursor = args.walk();
    args.children(&mut cursor)
        .any(|a| syntax.lambda.iter().any(|k| k == unwrap_argument(a, syntax).kind()))
}

/// 인자 노드 하나를 경로 조각들로 푼다.
///
/// 연결식은 왼쪽으로 중첩되므로 재귀로 내려간다. `"/a/" + x + "/b"`의 최상위
/// 왼쪽이 또 연결식이라, 한 겹만 보면 리터럴을 찾지 못해 호출을 통째로 놓친다.
fn url_template_of(
    node: Node,
    src: &[u8],
    syntax: &crate::rules::LangSyntax,
    constants: &std::collections::HashMap<String, String>,
) -> UrlTemplate {
    let mut parts = Vec::new();
    collect_parts(node, src, syntax, constants, &mut parts);
    UrlTemplate(parts)
}

// ANCHOR: collect_parts
fn collect_parts(
    node: Node,
    src: &[u8],
    syntax: &crate::rules::LangSyntax,
    constants: &std::collections::HashMap<String, String>,
    out: &mut Vec<UrlPart>,
) {
    if syntax.string.iter().any(|k| k == node.kind()) {
        out.extend(literal_parts(&strip_string_affixes(text(node, src))));
        return;
    }
    if syntax.concat.iter().any(|k| k == node.kind()) {
        if let (Some(left), Some(right)) = (
            node.child_by_field_name("left"),
            node.child_by_field_name("right"),
        ) {
            collect_parts(left, src, syntax, constants, out);
            collect_parts(right, src, syntax, constants, out);
            return;
        }
    }
    // `String.format("/api/orders/%s", id)` 처럼 형식 문자열로 만드는 경우
    // 첫 인자가 곧 경로다.
    if syntax.call.iter().any(|k| k == node.kind()) {
        if let Some(parts) = format_call_parts(node, src, syntax) {
            out.extend(parts);
            return;
        }
    }
    // 이름이면 값을 찾아본다. 같은 파일에 선언이 있으면 그 자리에서 값이 되고,
    // 없으면 이름을 남겨 두었다가 2패스에서 다른 파일의 상수로 채운다.
    if syntax.qualified_name.iter().any(|k| k == node.kind()) || node.kind() == "identifier" {
        let name = text(node, src);
        out.push(match constants.get(name) {
            Some(value) => UrlPart::Literal(value.clone()),
            None => UrlPart::Named(name.to_string()),
        });
        return;
    }
    out.push(UrlPart::Unknown);
}
// ANCHOR_END: collect_parts

/// 문자열 리터럴 하나를 조각으로 나눈다.
///
/// 리터럴이라고 전부 고정된 것은 아니다. 보간 문법이 문자열 안에 들어 있다.
/// 자바스크립트 템플릿 `` `/api/orders/${id}` ``, 파이썬 f-문자열
/// `f"/api/orders/{id}"`, C# 보간 `$"/api/orders/{id}"`가 그렇다.
/// Express나 react-router의 `:id` 표기도 같은 자리에 온다.
fn literal_parts(raw: &str) -> Vec<UrlPart> {
    let mut parts = Vec::new();
    let mut literal = String::new();
    let mut chars = raw.chars().peekable();
    let mut push_unknown = |literal: &mut String, parts: &mut Vec<UrlPart>| {
        if !literal.is_empty() {
            parts.push(UrlPart::Literal(std::mem::take(literal)));
        }
        parts.push(UrlPart::Unknown);
    };
    while let Some(c) = chars.next() {
        match c {
            '$' if chars.peek() == Some(&'{') => {
                chars.next();
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                }
                push_unknown(&mut literal, &mut parts);
            }
            '{' => {
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                }
                push_unknown(&mut literal, &mut parts);
            }
            // `/orders/:id` — 콜론 뒤 세그먼트 전체가 파라미터다.
            ':' if literal.ends_with('/') => {
                while chars.peek().is_some_and(|c| *c != '/') {
                    chars.next();
                }
                push_unknown(&mut literal, &mut parts);
            }
            _ => literal.push(c),
        }
    }
    if !literal.is_empty() {
        parts.push(UrlPart::Literal(literal));
    }
    parts
}

/// 형식 문자열의 자리 표시자를 조각으로 나눈다.
/// `%s`와 `%d`는 `String.format`이, `{}`와 `{0}`은 파이썬 `.format`과
/// C# 보간이 쓴다.
fn format_call_parts(
    node: Node,
    src: &[u8],
    syntax: &crate::rules::LangSyntax,
) -> Option<Vec<UrlPart>> {
    let args = node.child_by_field_name("arguments")?;
    let mut cursor = args.walk();
    let first = args
        .children(&mut cursor)
        .map(|a| unwrap_argument(a, syntax))
        .find(|a| syntax.string.iter().any(|k| k == a.kind()))?;
    let raw = strip_string_affixes(text(first, src));
    if !raw.starts_with('/') {
        return None;
    }
    let mut parts = Vec::new();
    let mut literal = String::new();
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '%' if chars.peek().is_some_and(|c| c.is_ascii_alphabetic()) => {
                chars.next();
                parts.push(UrlPart::Literal(std::mem::take(&mut literal)));
                parts.push(UrlPart::Unknown);
            }
            '{' => {
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                }
                parts.push(UrlPart::Literal(std::mem::take(&mut literal)));
                parts.push(UrlPart::Unknown);
            }
            _ => literal.push(c),
        }
    }
    if !literal.is_empty() {
        parts.push(UrlPart::Literal(literal));
    }
    Some(parts)
}

/// `index`번째 실인자에서 URL 문자열을 뽑는다.
fn url_argument_at(
    args: Node,
    src: &[u8],
    index: usize,
    syntax: &crate::rules::LangSyntax,
    constants: &std::collections::HashMap<String, String>,
) -> Option<UrlTemplate> {
    let mut cursor = args.walk();
    let mut position = 0usize;
    for arg in args.children(&mut cursor) {
        if matches!(arg.kind(), "(" | ")" | ",") {
            continue;
        }
        if position == index {
            return Some(url_template_of(
                unwrap_argument(arg, syntax),
                src,
                syntax,
                constants,
            ));
        }
        position += 1;
    }
    None
}

///
/// 이 선언을 감싸는 타입의 이름. `ApiPaths.ORDERS` 형태의 키를 만드는 데 쓴다.
fn enclosing_type_name(node: Node, src: &[u8]) -> Option<String> {
    let mut cur = node.parent();
    while let Some(n) = cur {
        if n.kind().contains("class_declaration")
            || n.kind().contains("interface_declaration")
            || n.kind().contains("object_declaration")
        {
            if let Some(name) = n.child_by_field_name("name") {
                return Some(text(name, src).to_string());
            }
        }
        cur = n.parent();
    }
    None
}

/// 파일 안에서 이름에 묶인 문자열 값을 모은다.
///
/// Java 백엔드는 경로를 `private static final String BASE = "/api/orders"`처럼
/// 상수로 빼는 관례가 강하다. 그 선언을 따라가지 않으면 호출 자리에 이름만
/// 남아 있어 경로를 알 수 없다.
///
/// 같은 파일 안에서만 찾는다. 이름은 짧고 흔해서(`BASE`, `API_URL`) 파일을
/// 넘나들며 치환하면 다른 파일의 값을 잘못 가져올 수 있다.
pub fn string_constants(
    root: Node,
    src: &[u8],
    syntax: &crate::rules::LangSyntax,
) -> std::collections::HashMap<String, String> {
    let mut table = std::collections::HashMap::new();
    if syntax.declaration.is_empty() {
        return table;
    }
    collect_declarations(root, src, syntax, &mut table);
    table
}

fn collect_declarations(
    node: Node,
    src: &[u8],
    syntax: &crate::rules::LangSyntax,
    table: &mut std::collections::HashMap<String, String>,
) {
    if syntax.declaration.iter().any(|k| k == node.kind()) {
        let name = node.child_by_field_name(&syntax.declaration_name_field);
        let value = node.child_by_field_name(&syntax.declaration_value_field);
        if let (Some(name), Some(value)) = (name, value) {
            // 선언된 값도 조각으로 푼다. 상수가 상수를 참조하는 경우까지
            // 따라가지는 않으므로, 리터럴만으로 이루어진 것만 표에 넣는다.
            let empty = std::collections::HashMap::new();
            let template = url_template_of(value, src, syntax, &empty);
            let text_out = template.render();
            if template.is_meaningful() && !template.has_unresolved() {
                // 같은 이름이 여러 번 선언되면 첫 번째를 쓴다. 재대입까지
                // 추적하려면 흐름 분석이 필요하고, 그만한 이득이 없다.
                let name = text(name, src).to_string();
                // `ApiPaths.ORDERS`로 참조할 수 있도록 클래스 이름을 붙인 키도
                // 함께 넣는다. 짧은 이름은 겹칠 수 있으나 한정 이름은 유일하다.
                if let Some(owner) = enclosing_type_name(node, src) {
                    table
                        .entry(format!("{owner}.{name}"))
                        .or_insert_with(|| text_out.clone());
                }
                table.entry(name).or_insert(text_out);
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_declarations(child, src, syntax, table);
    }
}

/// 문자열 리터럴의 접두와 따옴표를 벗긴다.
/// 파이썬의 `f"..."`, C#의 `$"..."`와 `@"..."`를 처리한다.
fn strip_string_affixes(raw: &str) -> String {
    raw.trim_start_matches(['f', 'F', 'r', 'R', 'b', 'B', 'u', 'U', '$', '@'])
        .trim_matches(['"', '\'', '`'])
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn java_tree(src: &str) -> (tree_sitter::Tree, Vec<u8>) {
        let mut p = Parser::new();
        p.set_language(&tree_sitter_java::LANGUAGE.into()).unwrap();
        (p.parse(src, None).unwrap(), src.as_bytes().to_vec())
    }

    fn ts_tree(src: &str) -> (tree_sitter::Tree, Vec<u8>) {
        let mut p = Parser::new();
        p.set_language(&tree_sitter_typescript::LANGUAGE_TSX.into()).unwrap();
        (p.parse(src, None).unwrap(), src.as_bytes().to_vec())
    }

    #[test]
    fn normalizes_all_three_param_syntaxes() {
        // 프런트–백엔드 매칭이 문자열 비교로 끝나야 한다.
        assert_eq!(normalize_route_path("/api/orders/{id}"), "/api/orders/{}");
        assert_eq!(normalize_route_path("/api/orders/:id"), "/api/orders/{}");
        assert_eq!(normalize_route_path("`/api/orders/${id}`"), "/api/orders/{}");
        assert_eq!(normalize_route_path("/api/orders/"), "/api/orders");
        assert_eq!(normalize_route_path("articles"), "/articles");
        assert_eq!(
            normalize_route_path("/api/users/{userId}/posts/{postId}"),
            "/api/users/{}/posts/{}"
        );
    }

    #[test]
    fn extracts_spring_routes_with_class_base_path() {
        let src = r#"
@RestController
@RequestMapping("/api/articles")
public class ArticleController {
    @GetMapping("/{slug}")
    public ArticleDto get(String slug) { return null; }

    @PostMapping
    public ArticleDto create(ArticleReq req) { return null; }

    @DeleteMapping("/{slug}/favorite")
    public void unfavorite(String slug) {}
}
"#;
        let (tree, bytes) = java_tree(src);
        let rules = FrameworkRules::effective(&FrameworkRules::default());
        let f = extract_annotated(tree.root_node(), &bytes, "java", &rules);

        let routes: Vec<(String, String, String)> = f
            .routes
            .iter()
            .map(|r| (r.method.clone(), r.path.clone(), r.handler.clone()))
            .collect();

        assert!(routes.contains(&("GET".into(), "/api/articles/{}".into(), "get".into())), "{routes:?}");
        assert!(routes.contains(&("POST".into(), "/api/articles".into(), "create".into())), "{routes:?}");
        assert!(
            routes.contains(&("DELETE".into(), "/api/articles/{}/favorite".into(), "unfavorite".into())),
            "{routes:?}"
        );
    }

    #[test]
    fn detects_bean_and_injections() {
        let src = r#"
@Service
@RequiredArgsConstructor
public class ArticleService {
    private final ArticleRepository articleRepository;
    @Autowired private TagService tagService;

    public ArticleService(UserRepository userRepository) {}
}
"#;
        let (tree, bytes) = java_tree(src);
        let rules = FrameworkRules::effective(&FrameworkRules::default());
        let f = extract_annotated(tree.root_node(), &bytes, "java", &rules);

        assert_eq!(f.beans.len(), 1);
        assert_eq!(f.beans[0].stereotype, "Service");

        let injected: Vec<&str> = f.injects.iter().map(|i| i.injected_type.as_str()).collect();
        // Lombok final 필드 · @Autowired 필드 · 생성자 파라미터 세 형태를 모두 잡아야 한다.
        assert!(injected.contains(&"ArticleRepository"), "{injected:?}");
        assert!(injected.contains(&"TagService"), "{injected:?}");
        assert!(injected.contains(&"UserRepository"), "{injected:?}");
    }

    #[test]
    fn request_mapping_method_argument_wins() {
        let src = r#"
@RestController
public class C {
    @RequestMapping(value = "/login", method = RequestMethod.POST)
    public void login() {}
}
"#;
        let (tree, bytes) = java_tree(src);
        let rules = FrameworkRules::effective(&FrameworkRules::default());
        let f = extract_annotated(tree.root_node(), &bytes, "java", &rules);
        assert_eq!(f.routes.len(), 1);
        assert_eq!(f.routes[0].method, "POST");
        assert_eq!(f.routes[0].path, "/login");
    }

    #[test]
    fn route_definitions_are_not_client_calls() {
        // miragejs / Express 스타일 라우트 등록은 클라이언트 호출이 아니다.
        let src = r#"
function makeServer() {
  this.post('/users/login', (schema, request) => { return 1; });
  app.get('/articles', handler);
  axios.get('/api/real');
}
"#;
        let (tree, bytes) = ts_tree(src);
        let rules = FrameworkRules::effective(&FrameworkRules::default());
        let calls = extract_api_calls(tree.root_node(), &bytes, "typescript", &rules);
        let paths: Vec<&str> = calls.iter().map(|c| c.path.as_str()).collect();
        assert_eq!(paths, vec!["/api/real"], "오탐: {paths:?}");
    }

    #[test]
    fn detects_dynamically_built_paths() {
        assert!(!has_dynamic_segment("/api/orders/${id}"));
        assert!(!has_dynamic_segment("/api/orders/${id}/items"));
        // 세그먼트 중간 치환 — 정적으로 결정 불가
        assert!(has_dynamic_segment("/users${isRegister ? '' : '/login'}"));
        assert!(has_dynamic_segment("/api/v${version}/orders"));
    }

    #[test]
    fn extracts_react_api_calls() {
        let src = r#"
export function useArticle(slug) {
  const a = fetch(`/api/articles/${slug}`);
  const b = axios.get('/api/tags');
  const c = api.post("/api/articles", body);
  const d = useState("");
  const e = items.map(x => x);
  return [a, b, c, d, e];
}
"#;
        let (tree, bytes) = ts_tree(src);
        let rules = FrameworkRules::effective(&FrameworkRules::default());
        let calls = extract_api_calls(tree.root_node(), &bytes, "typescript", &rules);
        let pairs: Vec<(String, String)> =
            calls.iter().map(|c| (c.method.clone(), c.path.clone())).collect();

        assert!(pairs.contains(&("GET".into(), "/api/articles/{}".into())), "{pairs:?}");
        assert!(pairs.contains(&("GET".into(), "/api/tags".into())), "{pairs:?}");
        assert!(pairs.contains(&("POST".into(), "/api/articles".into())), "{pairs:?}");
        // useState("")·map()은 HTTP 호출이 아니다.
        assert_eq!(pairs.len(), 3, "오탐: {pairs:?}");
    }
}

#[cfg(test)]
mod persistence_tests {
    use super::*;
    use tree_sitter::Parser;

    fn java(src: &str) -> FrameworkFacts {
        let mut p = Parser::new();
        p.set_language(&tree_sitter_java::LANGUAGE.into()).unwrap();
        let tree = p.parse(src, None).unwrap();
        let rules = FrameworkRules::effective(&FrameworkRules::default());
        extract_annotated(tree.root_node(), src.as_bytes(), "java", &rules)
    }

    #[test]
    fn sql_table_extraction_handles_mybatis_templates() {
        // MyBatis SQL은 #{param}·<if> 조각이 섞여 정식 파싱이 자주 실패한다.
        let t = tables_in_sql("SELECT * FROM orders o JOIN order_items i ON o.id = i.order_id WHERE o.id = #{id}");
        let names: Vec<&str> = t.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"orders"), "{t:?}");
        assert!(names.contains(&"order_items"), "{t:?}");

        let ins = tables_in_sql("INSERT INTO payments (id, amount) VALUES (#{id}, #{amt})");
        assert_eq!(ins, vec![("payments".to_string(), "insert".to_string())]);

        let upd = tables_in_sql("UPDATE dbo.customers SET name = #{name}");
        assert_eq!(upd[0].0, "customers", "스키마 접두는 벗긴다");

        // 예약어·짧은 토큰은 잡지 않는다.
        assert!(tables_in_sql("SELECT 1").is_empty());
    }

    #[test]
    fn jpa_entity_and_table() {
        let f = java(r#"
@Entity
@Table(name = "orders")
public class Order {
    private Long id;
}
"#);
        assert_eq!(f.entities.len(), 1);
        assert_eq!(f.entities[0].name, "Order");
        assert_eq!(f.entities[0].table.as_deref(), Some("orders"));
    }

    #[test]
    fn mybatis_annotation_mapper() {
        let f = java(r#"
@Mapper
public interface OrderMapper {
    @Select("SELECT * FROM orders WHERE id = #{id}")
    Order findById(Long id);

    @Insert("INSERT INTO orders (name) VALUES (#{name})")
    void save(Order order);
}
"#);
        // 인터페이스 메서드는 method_declaration 이 아닐 수 있으므로 테이블 참조로 확인한다.
        let tables: Vec<&str> = f.table_refs.iter().map(|t| t.table.as_str()).collect();
        assert!(tables.contains(&"orders"), "MyBatis 매퍼에서 테이블을 못 찾음: {:?}", f.table_refs);
    }
}

#[cfg(test)]
mod python_csharp_tests {
    use super::*;
    use tree_sitter::Parser;

    fn facts(lang_name: &str, ts: tree_sitter::Language, src: &str) -> FrameworkFacts {
        let mut p = Parser::new();
        p.set_language(&ts).unwrap();
        let tree = p.parse(src, None).unwrap();
        let rules = FrameworkRules::effective(&FrameworkRules::default());
        extract_annotated(tree.root_node(), src.as_bytes(), lang_name, &rules)
    }

    #[test]
    fn fastapi_and_flask_routes() {
        let f = facts("python", tree_sitter_python::LANGUAGE.into(), r#"
@app.get("/api/orders/{order_id}")
def get_order(order_id: int):
    return None

@router.post("/api/orders")
def create_order():
    return None

@app.route("/legacy", methods=["POST"])
def legacy():
    return None

@cache.get("/not-a-route")
def cached():
    return None
"#);
        let got: Vec<(String, String, String)> = f
            .routes
            .iter()
            .map(|r| (r.method.clone(), r.path.clone(), r.handler.clone()))
            .collect();

        assert!(got.contains(&("GET".into(), "/api/orders/{}".into(), "get_order".into())), "{got:?}");
        assert!(got.contains(&("POST".into(), "/api/orders".into(), "create_order".into())), "{got:?}");
        assert!(got.contains(&("POST".into(), "/legacy".into(), "legacy".into())), "Flask methods=: {got:?}");
        // @cache.get 은 라우트가 아니다
        assert!(!got.iter().any(|(_, p, _)| p.contains("not-a-route")), "오탐: {got:?}");
    }

    #[test]
    fn sqlalchemy_tablename() {
        let f = facts("python", tree_sitter_python::LANGUAGE.into(), r#"
class Order(Base):
    __tablename__ = "orders"
    id = Column(Integer, primary_key=True)
"#);
        assert_eq!(f.entities.len(), 1, "{:?}", f.entities);
        assert_eq!(f.entities[0].table.as_deref(), Some("orders"));
    }

    #[test]
    fn aspnet_attributes() {
        let f = facts("csharp", tree_sitter_c_sharp::LANGUAGE.into(), r#"
[ApiController]
[Route("api/orders")]
public class OrderController : ControllerBase
{
    [HttpGet("{id}")]
    public IActionResult GetOrder(int id) { return null; }

    [HttpPost]
    public IActionResult Create() { return null; }
}
"#);
        let got: Vec<(String, String)> =
            f.routes.iter().map(|r| (r.method.clone(), r.path.clone())).collect();
        assert!(got.contains(&("GET".into(), "/api/orders/{}".into())), "{got:?}");
        assert!(got.contains(&("POST".into(), "/api/orders".into())), "{got:?}");
        assert!(f.beans.iter().any(|b| b.name == "OrderController"), "{:?}", f.beans);
    }

    // ── HTTP 클라이언트 호출: 네 언어 ──────────────────────────────────
    // 백엔드도 다른 서비스를 부르므로 Java·C#·Python에서도 탐지되어야 한다.

    fn calls_in(lang: &str, language: tree_sitter::Language, src: &str) -> Vec<ApiCallFact> {
        let mut p = Parser::new();
        p.set_language(&language).unwrap();
        let tree = p.parse(src, None).unwrap();
        let rules = crate::rules::FrameworkRules::effective(&Default::default());
        extract_api_calls(tree.root_node(), src.as_bytes(), lang, &rules)
    }

    #[test]
    fn detects_api_calls_in_every_supported_language() {
        let cases: Vec<(&str, tree_sitter::Language, &str, &str, &str)> = vec![
            (
                "typescript",
                tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
                r#"axios.get("/api/articles");"#,
                "GET",
                "/api/articles",
            ),
            // JavaScript는 TypeScript 파서로 읽는다. lang.rs의 대응표가 그렇게 정해 두었다.
            (
                "javascript",
                tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
                r#"fetch("/api/articles");"#,
                "GET",
                "/api/articles",
            ),
            (
                "python",
                tree_sitter_python::LANGUAGE.into(),
                r#"requests.post("/api/articles")"#,
                "POST",
                "/api/articles",
            ),
            (
                "java",
                tree_sitter_java::LANGUAGE.into(),
                r#"class C { void m() { rest.getForObject("/api/articles", String.class); } }"#,
                "GET",
                "/api/articles",
            ),
            (
                "csharp",
                tree_sitter_c_sharp::LANGUAGE.into(),
                r#"class C { void M() { http.GetAsync("/api/articles"); } }"#,
                "GET",
                "/api/articles",
            ),
        ];
        for (lang, language, src, method, path) in cases {
            let calls = calls_in(lang, language, src);
            assert_eq!(calls.len(), 1, "{lang} 에서 호출을 찾지 못했다: {calls:?}");
            assert_eq!(calls[0].method, method, "{lang} 의 HTTP 메서드");
            assert_eq!(calls[0].path, path, "{lang} 의 경로");
        }
    }

    #[test]
    fn java_string_concat_becomes_path_parameter() {
        // `"/api/articles/" + slug` 는 세그먼트 하나가 통째로 치환되는 형태이므로
        // Spring 의 `/api/articles/{slug}` 정규화 결과와 같은 문자열이 되어야 한다.
        let calls = calls_in(
            "java",
            tree_sitter_java::LANGUAGE.into(),
            r#"class C { void m() { rest.getForObject("/api/articles/" + slug, String.class); } }"#,
        );
        assert_eq!(calls.len(), 1, "{calls:?}");
        assert_eq!(calls[0].path, "/api/articles/{}");
        assert!(!calls[0].dynamic, "세그먼트 전체 치환은 동적이 아니다");
    }

    #[test]
    fn java_concat_keeps_the_tail_after_the_parameter() {
        // `"/api/articles/" + slug + "/comments"` 는 트리에서 왼쪽으로 중첩되므로
        // 최상위 연결식의 왼쪽이 또 연결식이다. 왼쪽만 한 겹 보던 시절에는
        // 리터럴을 찾지 못해 이 호출을 통째로 놓쳤다.
        let calls = calls_in(
            "java",
            tree_sitter_java::LANGUAGE.into(),
            r#"class C { void m() { rest.getForObject("/api/articles/" + slug + "/comments", String.class); } }"#,
        );
        assert_eq!(calls.len(), 1, "{calls:?}");
        assert_eq!(calls[0].path, "/api/articles/{}/comments");
    }

    #[test]
    fn java_follows_constants_and_locals_in_the_same_file() {
        // Java 백엔드는 경로를 상수로 빼는 관례가 강하다. 선언을 따라가지
        // 않으면 호출 자리에 이름만 남아 경로를 알 수 없다.
        let calls = calls_in(
            "java",
            tree_sitter_java::LANGUAGE.into(),
            r#"
class Gateway {
    private static final String BASE = "/api/orders";
    void a() { rest.getForObject(BASE, List.class); }
    void b() { rest.getForObject(BASE + "/" + id, OrderDto.class); }
    void c() {
        String url = "/api/orders/summary";
        rest.getForObject(url, List.class);
    }
}"#,
        );
        let paths: Vec<&str> = calls.iter().map(|c| c.path.as_str()).collect();
        assert_eq!(
            paths,
            ["/api/orders", "/api/orders/{}", "/api/orders/summary"],
            "{calls:?}"
        );
    }

    #[test]
    fn string_format_becomes_a_path_parameter() {
        let calls = calls_in(
            "java",
            tree_sitter_java::LANGUAGE.into(),
            r#"class C { void m() { rest.getForObject(String.format("/api/orders/%s", id), O.class); } }"#,
        );
        assert_eq!(calls.len(), 1, "{calls:?}");
        assert_eq!(calls[0].path, "/api/orders/{}");
    }

    #[test]
    fn a_url_that_is_only_a_variable_is_skipped() {
        // 값이 어디서 오는지 알 수 없으면 잘못된 좌표를 주는 것보다 버리는 편이 낫다.
        let calls = calls_in(
            "java",
            tree_sitter_java::LANGUAGE.into(),
            r#"class C { void m() { rest.getForObject(props.getOrderUrl(), List.class); } }"#,
        );
        assert!(calls.is_empty(), "{calls:?}");
    }

    #[test]
    fn an_unknown_name_is_left_for_the_second_pass() {
        // 이 파일에 선언이 없는 이름은 값을 지우지 않고 `${이름}`으로 남긴다.
        // 경로 상수를 별도 파일에 모아 두는 관례가 흔해서, 인덱싱 2패스가
        // 전체 파일의 상수 표로 그것을 채운다.
        let calls = calls_in(
            "java",
            tree_sitter_java::LANGUAGE.into(),
            r#"class C { void m() { rest.getForObject(ApiPaths.ORDERS, List.class); } }"#,
        );
        assert_eq!(calls.len(), 1, "{calls:?}");
        assert_eq!(
            calls[0].template,
            UrlTemplate(vec![UrlPart::Named("ApiPaths.ORDERS".into())]),
            "이름을 잃으면 2패스에서 찾을 수 없다"
        );
        assert_eq!(calls[0].raw_path, "<ApiPaths.ORDERS>", "사람이 읽는 표기");
    }

    #[test]
    fn a_name_that_is_never_declared_becomes_unknown() {
        // 끝까지 선언을 못 찾으면 값을 알 수 없다. 그 자리는 `Unknown`이 되고,
        // 세그먼트를 통째로 차지하지 않으므로 동적으로 판정된다.
        let mut t = UrlTemplate(vec![
            UrlPart::Named("NOWHERE".into()),
            UrlPart::Literal("/items".into()),
        ]);
        t.fill(&std::collections::HashMap::new());
        assert_eq!(t, UrlTemplate(vec![UrlPart::Unknown, UrlPart::Literal("/items".into())]));
        assert_eq!(t.render(), "{}/items");
        assert!(t.is_dynamic(), "앞을 모르면 어떤 엔드포인트인지 알 수 없다");
    }

    #[test]
    fn a_parameter_that_fills_a_whole_segment_is_not_dynamic() {
        // `"/api/orders/" + id` 는 Spring 의 `/{id}` 가 정규화된 것과 같은
        // 문자열이 되므로 라우트에 연결할 수 있다.
        let whole = UrlTemplate(vec![
            UrlPart::Literal("/api/orders/".into()),
            UrlPart::Unknown,
        ]);
        assert_eq!(whole.render(), "/api/orders/{}");
        assert!(!whole.is_dynamic());

        // `"/api/orders" + suffix` 는 무엇이 붙느냐에 따라 경로가 달라진다.
        let partial = UrlTemplate(vec![
            UrlPart::Literal("/api/orders".into()),
            UrlPart::Unknown,
        ]);
        assert_eq!(partial.render(), "/api/orders{}");
        assert!(partial.is_dynamic());
    }

    #[test]
    fn java_concat_without_slash_is_dynamic_after_the_second_pass() {
        // 슬래시 없이 이어 붙이면 어떤 엔드포인트가 되는지 알 수 없다.
        //
        // 다만 1패스에서는 아직 판정하지 않는다. `suffix`가 다른 파일에 선언된
        // 상수일 수도 있기 때문이다. 2패스에서 값을 못 찾으면 그때 `Unknown`이
        // 되고 동적으로 확정된다.
        let calls = calls_in(
            "java",
            tree_sitter_java::LANGUAGE.into(),
            r#"class C { void m() { rest.getForObject("/api/article" + suffix, String.class); } }"#,
        );
        assert_eq!(calls.len(), 1, "{calls:?}");
        assert!(calls[0].template.has_unresolved(), "이름이 남아 있어야 한다");

        let mut t = calls[0].template.clone();
        t.fill(&std::collections::HashMap::new());
        assert!(t.is_dynamic(), "값을 못 찾았으므로 경로를 확정할 수 없다");
        assert_eq!(t.render(), "/api/article{}");
    }

    #[test]
    fn python_fstring_and_csharp_interpolation_are_normalized() {
        let py = calls_in(
            "python",
            tree_sitter_python::LANGUAGE.into(),
            r#"requests.get(f"/api/articles/{slug}")"#,
        );
        assert_eq!(py.len(), 1, "{py:?}");
        assert_eq!(py[0].path, "/api/articles/{}");

        let cs = calls_in(
            "csharp",
            tree_sitter_c_sharp::LANGUAGE.into(),
            r#"class C { void M() { http.GetAsync($"/api/articles/{slug}"); } }"#,
        );
        assert_eq!(cs.len(), 1, "{cs:?}");
        assert_eq!(cs[0].path, "/api/articles/{}");
    }

    #[test]
    fn java_map_put_is_not_an_api_call() {
        // `Map.put` 은 이름이 겹치지만 HTTP 호출이 아니다.
        // 기본 규칙에서 put 을 뺀 이유를 고정한다.
        let calls = calls_in(
            "java",
            tree_sitter_java::LANGUAGE.into(),
            r#"class C { void m() { routes.put("/api/articles", handler); } }"#,
        );
        assert!(calls.is_empty(), "Map.put 을 API 호출로 오인했다: {calls:?}");
    }

    #[test]
    fn lambda_argument_is_route_registration_in_every_language() {
        // 인자에 람다가 있으면 핸들러 등록이다. 언어마다 람다 노드 이름이 다르므로
        // 네 언어에서 모두 걸러지는지 고정한다.
        let java = calls_in(
            "java",
            tree_sitter_java::LANGUAGE.into(),
            r#"class C { void m() { rest.getForObject("/api/x", r -> r); } }"#,
        );
        assert!(java.is_empty(), "{java:?}");

        let py = calls_in(
            "python",
            tree_sitter_python::LANGUAGE.into(),
            r#"app.get("/api/x", lambda r: r)"#,
        );
        assert!(py.is_empty(), "{py:?}");

        let cs = calls_in(
            "csharp",
            tree_sitter_c_sharp::LANGUAGE.into(),
            r#"class C { void M() { http.GetAsync("/api/x", r => r); } }"#,
        );
        assert!(cs.is_empty(), "{cs:?}");
    }
}

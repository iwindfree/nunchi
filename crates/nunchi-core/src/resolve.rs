//! 이름 기반 참조 해소 — 빠른 경로 (PLAN.md 3.9절)
//!
//! 여기서 나오는 해소율이 `nunchi doctor`의 핵심 지표다. 이 값이 낮으면
//! `CALLS` 엣지가 비어 **그래프가 사실상 파일 목록으로 전락한다**.
//! 정밀 해소는 SCIP 경로(Phase 1b)가 맡으며, 여기 결과는 모두 `Provenance::Fast`다.

use crate::model::NodeId;
use std::collections::HashMap;

/// 이름 → 심볼 후보들. 같은 이름이 여러 곳에 정의될 수 있다.
#[derive(Debug, Default)]
pub struct SymbolTable {
    by_name: HashMap<String, Vec<NodeId>>,
    /// 심볼 ID → 종류(`method`, `field` 등). 필드를 검증 대상에서 빼는 데 쓴다.
    kinds: HashMap<String, String>,
    /// `com/example/OrderService.java` 같은 경로 → 파일 노드
    by_path: HashMap<String, NodeId>,
    /// 상위 타입 이름 → 하위 타입 이름들. 인터페이스→구현 해소용.
    implementors: HashMap<String, Vec<String>>,
}

/// 후보가 이보다 많으면 해소를 포기한다. `get`·`build` 같은 흔한 이름이
/// 그래프를 오염시키는 것을 막는다.
const MAX_CANDIDATES: usize = 3;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ResolveStats {
    pub total: usize,
    /// 후보가 정확히 하나 — 신뢰할 수 있다
    pub resolved: usize,
    /// 후보가 2~3개 — 낮은 confidence로 모두 연결한다
    pub ambiguous: usize,
    /// 후보 없음 — 대개 외부 라이브러리다
    pub unresolved: usize,
    /// 후보가 너무 많아 포기
    pub dropped: usize,
}

/// 미해소 호출 이름 집계. 사람이 "외부 API인가, 우리 결함인가"를 판단하는 근거다.
#[derive(Debug, Default, Clone)]
pub struct UnresolvedTally(std::collections::HashMap<String, usize>);

impl UnresolvedTally {
    pub fn record(&mut self, name: &str) {
        *self.0.entry(name.to_string()).or_default() += 1;
    }

    /// 빈도 상위 `n`개.
    pub fn top(&self, n: usize) -> Vec<(String, usize)> {
        let mut v: Vec<_> = self.0.iter().map(|(k, c)| (k.clone(), *c)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v.truncate(n);
        v
    }
}

impl ResolveStats {
    /// 호출 **연결률** — 엣지가 생긴 호출의 비율.
    ///
    /// 주의: 분모에 외부 라이브러리 호출(std의 `push`, React의 `useState` 등)이
    /// 그대로 들어간다. 따라서 이 값 자체에 "95%" 같은 목표를 걸 수 없다.
    /// 판단은 `top_unresolved`와 함께 해야 한다 — 미해소 이름이 외부 API면 정상이고,
    /// 내부 심볼이어야 할 이름이면 추출기 결함이다.
    /// 계획서의 95% 목표는 SCIP 정밀 경로(Phase 1b) 지표다.
    pub fn rate(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        (self.resolved + self.ambiguous) as f32 / self.total as f32
    }
}

impl SymbolTable {
    pub fn insert_symbol(&mut self, name: &str, id: NodeId, kind: &str) {
        self.kinds.insert(id.0.clone(), kind.to_string());
        self.by_name.entry(name.to_string()).or_default().push(id);
    }

    /// 호출 대상으로 의미 있는 단위인지. 필드·프로퍼티는 제외한다 —
    /// Lombok 빌더(`.body(...)`)나 접근자 호출이 전부 "검증 대상"으로 잡히면
    /// TESTS 엣지가 잡음으로 뒤덮인다(실측에서 628건 중 대부분이 그랬다).
    pub fn is_callable_unit(&self, id: &NodeId) -> bool {
        matches!(
            self.kinds.get(&id.0).map(String::as_str),
            Some("method" | "function" | "constructor" | "class" | "record")
        )
    }

    /// `OrderServiceTest` → `OrderService` 처럼 이름으로 검증 대상을 찾는다.
    /// 호출 기반보다 정밀도가 훨씬 높다.
    pub fn subject_of_test(&self, test_name: &str) -> Vec<NodeId> {
        let base = test_name
            .strip_suffix("Tests")
            .or_else(|| test_name.strip_suffix("Test"))
            .or_else(|| test_name.strip_suffix("Spec"))
            .or_else(|| test_name.strip_prefix("Test"))
            .unwrap_or("");
        if base.len() < 3 {
            return Vec::new();
        }
        self.candidates(base)
            .iter()
            .filter(|id| self.is_callable_unit(id))
            .cloned()
            .collect()
    }

    /// 심볼이 테스트 코드에 정의됐는지. NodeId에 경로가 들어 있어 그것으로 판단한다.
    pub fn is_test_symbol(&self, id: &NodeId) -> bool {
        crate::index::is_test_path(id.as_str())
    }

    pub fn insert_supertype(&mut self, sub: &str, sup: &str) {
        self.implementors
            .entry(sup.to_string())
            .or_default()
            .push(sub.to_string());
    }

    /// 주입 대상 해소. `@Autowired OrderService`(인터페이스)를 실제 구현체로 잇는다.
    ///
    /// 인터페이스 자체도 후보로 남긴다 — 구현이 여러 개일 때 인터페이스가
    /// 공통 진입점 역할을 하기 때문이다.
    pub fn resolve_injection(&self, type_name: &str) -> Vec<NodeId> {
        let mut out: Vec<NodeId> = self.candidates(type_name).to_vec();
        if let Some(impls) = self.implementors.get(type_name) {
            for name in impls {
                out.extend(self.candidates(name).iter().cloned());
            }
        }
        out.sort();
        out.dedup();
        out
    }

    pub fn insert_file(&mut self, path: &str, id: NodeId) {
        self.by_path.insert(crate::path::compare_key(path), id);
    }

    /// 후보 목록. 비었으면 외부 참조로 본다.
    pub fn candidates(&self, name: &str) -> &[NodeId] {
        self.by_name.get(name).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// 호출 하나를 해소한다. 반환값은 `(대상, confidence)` 목록.
    pub fn resolve_call(
        &self,
        callee: &str,
        stats: &mut ResolveStats,
        tally: &mut UnresolvedTally,
    ) -> Vec<(NodeId, f32)> {
        stats.total += 1;
        let candidates = self.candidates(callee);
        match candidates.len() {
            0 => {
                stats.unresolved += 1;
                tally.record(callee);
                Vec::new()
            }
            1 => {
                stats.resolved += 1;
                // 이름 일치일 뿐 타입 해소가 아니므로 1.0을 주지 않는다.
                vec![(candidates[0].clone(), 0.8)]
            }
            n if n <= MAX_CANDIDATES => {
                stats.ambiguous += 1;
                let confidence = 0.8 / n as f32;
                candidates.iter().cloned().map(|id| (id, confidence)).collect()
            }
            _ => {
                stats.dropped += 1;
                tally.record(callee);
                Vec::new()
            }
        }
    }

    /// import 경로를 저장소 내 파일로 해소한다. 실패하면 외부 의존성이다.
    pub fn resolve_import(&self, lang: &str, from_file: &str, spec: &str) -> Option<NodeId> {
        let candidate_paths = match lang {
            "java" => java_import_paths(spec),
            "typescript" | "javascript" => ts_import_paths(from_file, spec)?,
            _ => return None,
        };
        candidate_paths
            .iter()
            .find_map(|p| self.by_path.get(&crate::path::compare_key(p)).cloned())
    }
}

/// `com.example.order.OrderService` → `com/example/order/OrderService.java`
fn java_import_paths(spec: &str) -> Vec<String> {
    if spec.ends_with(".*") {
        return Vec::new();
    }
    let as_path = spec.replace('.', "/");
    // Maven/Gradle 표준 레이아웃을 함께 시도한다.
    vec![
        format!("{as_path}.java"),
        format!("src/main/java/{as_path}.java"),
    ]
}

/// `./useOrder`, `../api/client` → 현재 파일 기준 상대 경로 후보들
fn ts_import_paths(from_file: &str, spec: &str) -> Option<Vec<String>> {
    if !spec.starts_with('.') {
        return None; // 패키지 import — 외부 의존성
    }
    let dir = from_file.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let joined = normalize_relative(dir, spec)?;
    const EXTS: &[&str] = &["ts", "tsx", "js", "jsx"];
    let mut out = Vec::new();
    for ext in EXTS {
        out.push(format!("{joined}.{ext}"));
        out.push(format!("{joined}/index.{ext}"));
    }
    Some(out)
}

/// `a/b` + `../c` → `a/c`
fn normalize_relative(dir: &str, spec: &str) -> Option<String> {
    let mut parts: Vec<&str> = if dir.is_empty() {
        Vec::new()
    } else {
        dir.split('/').collect()
    };
    for segment in spec.split('/') {
        match segment {
            "." | "" => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    Some(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tally_ranks_by_frequency() {
        let mut t = UnresolvedTally::default();
        for _ in 0..3 { t.record("unwrap"); }
        t.record("push");
        assert_eq!(t.top(1), vec![("unwrap".to_string(), 3)]);
    }

    #[test]
    fn single_candidate_resolves() {
        let mut table = SymbolTable::default();
        table.insert_symbol("findOne", NodeId("sym:api/A.java#findOne".into()), "method");
        let mut stats = ResolveStats::default();
        let hits = table.resolve_call("findOne", &mut stats, &mut UnresolvedTally::default());
        assert_eq!(hits.len(), 1);
        assert_eq!(stats.resolved, 1);
        assert_eq!(stats.rate(), 1.0);
    }

    #[test]
    fn unknown_names_count_as_unresolved() {
        let table = SymbolTable::default();
        let mut stats = ResolveStats::default();
        assert!(table.resolve_call("println", &mut stats, &mut UnresolvedTally::default()).is_empty());
        assert_eq!(stats.unresolved, 1);
        assert_eq!(stats.rate(), 0.0);
    }

    #[test]
    fn too_many_candidates_are_dropped() {
        let mut table = SymbolTable::default();
        for i in 0..10 {
            table.insert_symbol("get", NodeId(format!("sym:{i}")), "method");
        }
        let mut stats = ResolveStats::default();
        assert!(table.resolve_call("get", &mut stats, &mut UnresolvedTally::default()).is_empty());
        assert_eq!(stats.dropped, 1);
    }

    #[test]
    fn injection_resolves_through_implementors() {
        let mut t = SymbolTable::default();
        t.insert_symbol("OrderService", NodeId("sym:api/OrderService.java#OrderService".into()), "interface");
        t.insert_symbol("OrderServiceImpl", NodeId("sym:api/OrderServiceImpl.java#OrderServiceImpl".into()), "class");
        t.insert_supertype("OrderServiceImpl", "OrderService");

        // 인터페이스를 주입받으면 인터페이스와 구현체가 모두 후보다.
        let hits = t.resolve_injection("OrderService");
        assert_eq!(hits.len(), 2, "구현체까지 이어져야 한다: {hits:?}");

        // 상속 관계가 없는 타입은 자기 자신만.
        t.insert_symbol("Standalone", NodeId("sym:api/S.java#Standalone".into()), "class");
        assert_eq!(t.resolve_injection("Standalone").len(), 1);
    }

    #[test]
    fn test_subject_is_found_by_name_and_fields_excluded() {
        let mut t = SymbolTable::default();
        t.insert_symbol("OrderService", NodeId("sym:api/OrderService.java#OrderService".into()), "class");
        t.insert_symbol("body", NodeId("sym:api/Dto.java#body".into()), "field");

        assert_eq!(t.subject_of_test("OrderServiceTest").len(), 1);
        assert_eq!(t.subject_of_test("TestOrderService").len(), 1);
        // 필드는 검증 대상이 아니다 — Lombok 빌더 호출이 전부 걸리는 것을 막는다.
        assert!(!t.is_callable_unit(&NodeId("sym:api/Dto.java#body".into())));
        // 너무 짧은 접두는 오탐이 된다.
        assert!(t.subject_of_test("AbTest").is_empty());
    }

    #[test]
    fn java_imports_map_to_paths() {
        let mut table = SymbolTable::default();
        table.insert_file(
            "src/main/java/com/example/OrderService.java",
            NodeId("file:api/OrderService".into()),
        );
        let hit = table.resolve_import("java", "src/main/java/com/example/App.java", "com.example.OrderService");
        assert!(hit.is_some());
    }

    #[test]
    fn ts_relative_imports_resolve() {
        let mut table = SymbolTable::default();
        table.insert_file("src/hooks/useOrder.ts", NodeId("file:web/useOrder".into()));
        let hit = table.resolve_import("typescript", "src/pages/Order.tsx", "../hooks/useOrder");
        assert!(hit.is_some());
    }

    #[test]
    fn bare_package_imports_are_external() {
        let table = SymbolTable::default();
        assert!(table.resolve_import("typescript", "src/a.ts", "react").is_none());
    }
}

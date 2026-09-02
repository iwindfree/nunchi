//! MCP 서버 — 에이전트가 그래프에 질의하는 통로 (PLAN.md 3절)
//!
//! **툴 개수를 최소로 유지한다.** 스키마는 대화 시작 시점부터 상시 비용이며,
//! Uber가 MCP 게이트웨이로 줄이려 한 것이 정확히 이 비용이다(PLAN.md 0절).
//! 그래서 5개만 노출하고, 배칭이 필요하면 같은 기능의 CLI를 쓰게 한다.

use anyhow::Result;
use nunchi_core::graph::MemGraph;
use nunchi_core::model::{Direction, EdgeKind, NodeId};
use nunchi_core::store::Store;
use nunchi_core::{pack, Config, SqliteStore};
use rmcp::handler::server::ServerHandler;
use rmcp::model::*;
use rmcp::service::{RequestContext, RoleServer, ServiceExt};
use rmcp::ErrorData as McpError;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub struct NunchiServer {
    config: Config,
    db_path: PathBuf,
    roots: HashMap<String, PathBuf>,
}

impl NunchiServer {
    pub fn new(config: Config, db_path: PathBuf) -> Self {
        let roots = pack::repo_roots(&config);
        NunchiServer { config, db_path, roots }
    }

    fn store(&self) -> Result<SqliteStore, McpError> {
        SqliteStore::open(&self.db_path)
            .map_err(|e| McpError::internal_error(format!("인덱스를 열 수 없습니다: {e}"), None))
    }
}

fn schema(props: serde_json::Value, required: &[&str]) -> Arc<JsonObject> {
    let value = serde_json::json!({
        "type": "object",
        "properties": props,
        "required": required,
    });
    let obj = value.as_object().cloned().unwrap_or_default();
    Arc::new(obj)
}

fn tool(name: &'static str, description: &'static str, input_schema: Arc<JsonObject>) -> Tool {
    Tool::new(Cow::Borrowed(name), Cow::Borrowed(description), input_schema)
}

fn ok_json(value: serde_json::Value) -> CallToolResponse {
    let text = serde_json::to_string(&value).unwrap_or_else(|_| "{}".into());
    CallToolResult::success(vec![ContentBlock::text(text)]).into()
}

fn arg_str(req: &CallToolRequestParams, key: &str) -> Result<String, McpError> {
    req.arguments
        .as_ref()
        .and_then(|a| a.get(key))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| McpError::invalid_params(format!("`{key}` 인자가 필요합니다"), None))
}

fn arg_usize(req: &CallToolRequestParams, key: &str, default: usize) -> usize {
    req.arguments
        .as_ref()
        .and_then(|a| a.get(key))
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(default)
}

impl ServerHandler for NunchiServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.instructions = Some(
                "코드베이스 컨텍스트 그래프. 파일을 대량으로 읽기 전에 nunchi_pack을 먼저 \
                 호출하세요. 반환값은 답이 아니라 좌표(path:line)이므로, 필요한 범위만 \
                 Read하면 됩니다."
                .into(),
        );
        info
    }

    async fn list_tools(
        &self,
        _params: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let tools = vec![
                tool(
                    "nunchi_pack",
                    "태스크 설명으로 컨텍스트 팩을 만든다. 토큰 예산 안에서 랭킹된 심볼 \
                     스켈레톤과 정확한 좌표(path:line)를 반환한다. 코드 탐색 전 첫 호출로 쓴다.",
                    schema(
                        serde_json::json!({
                            "task": {"type": "string", "description": "무엇을 하려는지 한 문장"},
                            "budget": {"type": "integer", "description": "토큰 예산 (기본 4000)"}
                        }),
                        &["task"],
                    ),
                ),
                tool(
                    "nunchi_find",
                    "심볼·파일·라우트를 이름으로 찾아 좌표를 반환한다.",
                    schema(
                        serde_json::json!({
                            "query": {"type": "string"},
                            "limit": {"type": "integer"}
                        }),
                        &["query"],
                    ),
                ),
                tool(
                    "nunchi_neighbors",
                    "노드의 이웃(호출자·피호출자·구현·주입)을 반환한다.",
                    schema(
                        serde_json::json!({
                            "id": {"type": "string", "description": "nunchi_find가 반환한 노드 id"},
                            "depth": {"type": "integer"},
                            "kinds": {"type": "array", "items": {"type": "string"},
                                      "description": "calls, injects, imports, handles, calls_api 등"}
                        }),
                        &["id"],
                    ),
                ),
                tool(
                    "nunchi_impact",
                    "이 심볼을 고치면 무엇이 깨지는지. 전이 참조·테스트·교차 저장소 연결을 \
                     함께 반환한다.",
                    schema(serde_json::json!({"id": {"type": "string"}}), &["id"]),
                ),
                tool(
                    "nunchi_doctor",
                    "인덱스 상태 — 커버리지, 연결률, 교차 저장소 연결 수.",
                    schema(serde_json::json!({}), &[]),
                ),
        ];
        Ok(ListToolsResult { tools, ..Default::default() })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let store = self.store()?;
        let err = |e: anyhow::Error| McpError::internal_error(e.to_string(), None);

        match request.name.as_ref() {
            "nunchi_pack" => {
                let task = arg_str(&request, "task")?;
                let budget = arg_usize(&request, "budget", 4000);
                let graph = MemGraph::load(&store).map_err(err)?;
                let opts = pack::PackOptions {
                    budget,
                    weights: self.config.rank,
                    ..Default::default()
                };
                let result =
                    pack::build_pack(&store, &graph, &task, &self.roots, &opts).map_err(err)?;
                Ok(ok_json(serde_json::to_value(result).unwrap_or_default()))
            }
            "nunchi_find" => {
                let query = arg_str(&request, "query")?;
                let limit = arg_usize(&request, "limit", 20);
                let hits = store.search(&query, limit).map_err(err)?;
                let items: Vec<_> = hits
                    .iter()
                    .map(|h| {
                        serde_json::json!({
                            "id": h.node.id.0,
                            "ref": h.node.reference(),
                            "sym": h.node.name,
                            "kind": h.node.kind.as_str(),
                            "repo": h.node.repo,
                            "sig": h.node.signature,
                            "score": h.score,
                        })
                    })
                    .collect();
                Ok(ok_json(serde_json::json!({ "items": items })))
            }
            "nunchi_neighbors" => {
                let id = NodeId(arg_str(&request, "id")?);
                let depth = arg_usize(&request, "depth", 1) as u32;
                let kinds: Vec<EdgeKind> = request
                    .arguments
                    .as_ref()
                    .and_then(|a| a.get("kinds"))
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().and_then(EdgeKind::parse))
                            .collect()
                    })
                    .unwrap_or_default();
                let nodes = store
                    .neighbors(&id, &kinds, Direction::Both, depth)
                    .map_err(err)?;
                Ok(ok_json(serde_json::json!({
                    "items": nodes.iter().map(|n| serde_json::json!({
                        "id": n.id.0, "ref": n.reference(), "sym": n.name,
                        "kind": n.kind.as_str(), "repo": n.repo, "sig": n.signature,
                    })).collect::<Vec<_>>()
                })))
            }
            "nunchi_impact" => {
                let id = NodeId(arg_str(&request, "id")?);
                let callers = store
                    .neighbors(&id, &[EdgeKind::Calls, EdgeKind::Injects], Direction::In, 2)
                    .map_err(err)?;
                let cross = store
                    .neighbors(
                        &id,
                        &[EdgeKind::Handles, EdgeKind::CallsApi],
                        Direction::Both,
                        2,
                    )
                    .map_err(err)?;
                let brief = |n: &nunchi_core::Node| {
                    serde_json::json!({
                        "id": n.id.0, "ref": n.reference(), "sym": n.name,
                        "kind": n.kind.as_str(), "repo": n.repo,
                    })
                };
                Ok(ok_json(serde_json::json!({
                    "callers": callers.iter().take(40).map(brief).collect::<Vec<_>>(),
                    "cross_repo": cross.iter().take(20).map(brief).collect::<Vec<_>>(),
                })))
            }
            "nunchi_doctor" => {
                let metrics: serde_json::Value = store
                    .get_meta("metrics")
                    .map_err(err)?
                    .and_then(|m| serde_json::from_str(&m).ok())
                    .unwrap_or(serde_json::Value::Null);
                Ok(ok_json(serde_json::json!({
                    "solution": self.config.solution.name,
                    "nodes": store.count_nodes().map_err(err)?,
                    "edges": store.count_edges().map_err(err)?,
                    "metrics": metrics,
                })))
            }
            other => Err(McpError::invalid_params(
                format!("알 수 없는 툴: {other}"),
                None,
            )),
        }
    }
}

/// stdio로 MCP 서버를 띄운다.
pub fn run(config: Config, db_path: PathBuf) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(async move {
        let service = NunchiServer::new(config, db_path)
            .serve(rmcp::transport::io::stdio())
            .await?;
        service.waiting().await?;
        Ok::<_, anyhow::Error>(())
    })
}

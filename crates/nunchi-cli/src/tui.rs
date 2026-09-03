//! TUI — 임베디드 저장소를 고르며 잃은 시각화를 되찾는 자리 (PLAN.md 1.6절)
//!
//! **설계 규칙: TUI는 CLI가 이미 내놓는 데이터의 뷰어일 뿐, 고유 기능을 갖지 않는다.**
//! TUI에서만 할 수 있는 일이 생기는 순간 자동화가 막히고 온보딩이 수작업이 된다
//! (PLAN.md 3.8절).
//!
//! 존재 이유는 진단이다. 에이전트가 헛다리를 짚었을 때 원인이 *추출 실패*인지
//! *랭킹 오류*인지 *인덱스 노후*인지 사람이 갈라낼 수단이 필요하다.

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use nunchi_core::graph::MemGraph;
use nunchi_core::model::{Direction, EdgeKind, NodeId};
use nunchi_core::store::Store;
use nunchi_core::{pack, Config, SqliteStore};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Row, Table, Tabs};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Explore,
    Impact,
    Index,
    Pack,
    Bench,
}

impl Screen {
    const ALL: [Screen; 5] = [
        Screen::Explore,
        Screen::Impact,
        Screen::Index,
        Screen::Pack,
        Screen::Bench,
    ];
    fn title(self) -> &'static str {
        match self {
            Screen::Explore => "① 탐색",
            Screen::Impact => "② 영향범위",
            Screen::Index => "③ 인덱스",
            Screen::Pack => "④ 팩 미리보기",
            Screen::Bench => "⑤ 지표",
        }
    }
}

/// 조정 대상 가중치. `s`를 누르면 nunchi.toml에 저장된다.
const WEIGHT_LABELS: [&str; 5] = ["α bm25", "β ppr", "γ recency", "δ cochange", "ε central"];

struct App {
    config: Config,
    config_path: PathBuf,
    db_path: PathBuf,
    store: SqliteStore,
    graph: MemGraph,
    roots: HashMap<String, PathBuf>,
    metrics: serde_json::Value,

    screen: Screen,
    input: String,
    editing: bool,
    status: String,

    results: Vec<ResultRow>,
    list_state: ListState,
    pack: Option<pack::Pack>,
    budget: usize,
    weight_cursor: usize,
    dirty_weights: bool,
}

struct ResultRow {
    id: NodeId,
    label: String,
    reference: String,
}

impl App {
    fn new(config: Config, config_path: PathBuf, db_path: PathBuf) -> Result<Self> {
        let store = SqliteStore::open(&db_path)?;
        let graph = MemGraph::load(&store)?;
        let roots = pack::repo_roots(&config);
        let metrics: serde_json::Value = store
            .get_meta("metrics")?
            .and_then(|m| serde_json::from_str(&m).ok())
            .unwrap_or(serde_json::Value::Null);
        Ok(App {
            config,
            config_path,
            db_path,
            store,
            graph,
            roots,
            metrics,
            screen: Screen::Explore,
            input: String::new(),
            editing: true,
            status: "질의를 입력하고 Enter. [tab] 화면 전환 · [q] 종료".into(),
            results: Vec::new(),
            list_state: ListState::default(),
            pack: None,
            budget: 4000,
            weight_cursor: 0,
            dirty_weights: false,
        })
    }

    /// 가중치를 델타만큼 조정한다. 0~2로 제한한다.
    fn adjust_weight(&mut self, index: usize, delta: f32) {
        let w = &mut self.config.rank;
        let slot = match index {
            0 => &mut w.alpha_bm25,
            1 => &mut w.beta_ppr,
            2 => &mut w.gamma_recency,
            3 => &mut w.delta_cochange,
            _ => &mut w.epsilon_central,
        };
        *slot = (*slot + delta).clamp(0.0, 2.0);
    }

    fn weight_values(&self) -> [f32; 5] {
        let w = &self.config.rank;
        [
            w.alpha_bm25,
            w.beta_ppr,
            w.gamma_recency,
            w.delta_cochange,
            w.epsilon_central,
        ]
    }

    fn run_query(&mut self) {
        match self.screen {
            Screen::Pack => self.rebuild_pack(),
            Screen::Impact => self.run_impact(),
            _ => self.run_search(),
        }
    }

    fn run_search(&mut self) {
        match self.store.search(&self.config.semantic.expand_query(&self.input), 200) {
            Ok(hits) => {
                self.results = hits
                    .iter()
                    .map(|h| ResultRow {
                        id: h.node.id.clone(),
                        label: format!(
                            "{:>6.2}  {:<10} {}",
                            h.score,
                            h.node.kind.as_str(),
                            h.node.name
                        ),
                        reference: h.node.reference().unwrap_or_default(),
                    })
                    .collect();
                self.list_state.select(if self.results.is_empty() { None } else { Some(0) });
                self.status = format!("{}건", self.results.len());
            }
            Err(e) => self.status = format!("검색 실패: {e}"),
        }
    }

    fn run_impact(&mut self) {
        let Some(selected) = self.selected_id() else {
            self.status = "① 탐색에서 항목을 먼저 고르세요".into();
            return;
        };
        let callers = self.store.neighbors(
            &selected,
            &[EdgeKind::Calls, EdgeKind::Injects, EdgeKind::CallsApi, EdgeKind::Handles],
            Direction::Both,
            2,
        );
        match callers {
            Ok(nodes) => {
                self.results = nodes
                    .iter()
                    .map(|n| ResultRow {
                        id: n.id.clone(),
                        label: format!("{:<10} [{}] {}", n.kind.as_str(), n.repo, n.name),
                        reference: n.reference().unwrap_or_default(),
                    })
                    .collect();
                self.status = format!("영향 범위 {}건", self.results.len());
            }
            Err(e) => self.status = format!("실패: {e}"),
        }
    }

    fn selected_id(&self) -> Option<NodeId> {
        self.list_state
            .selected()
            .and_then(|i| self.results.get(i))
            .map(|r| r.id.clone())
    }

    fn rebuild_pack(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }
        let opts = pack::PackOptions {
            budget: self.budget,
            weights: self.config.rank,
            synonyms: self.config.semantic.clone(),
            ..Default::default()
        };
        match pack::build_pack(&self.store, &self.graph, &self.input, &self.roots, &opts) {
            Ok(p) => {
                self.status = format!("used {}/{}", p.used, p.budget);
                self.pack = Some(p);
            }
            Err(e) => self.status = format!("팩 생성 실패: {e}"),
        }
    }

    fn save_weights(&mut self) {
        // 가중치는 **공용 파일**로 저장한다. 저장소에 커밋되어 양쪽 머신이
        // 같은 값을 쓰게 하기 위해서다(PLAN.md 3.10절). 머신별 nunchi.toml에
        // 넣으면 경로가 섞여 gitignore 대상이 되고 공유가 불가능해진다.
        let dir = self
            .config_path
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf();
        match self.config.save_shared(&dir) {
            Ok(path) => {
                self.dirty_weights = false;
                self.status = format!(
                    "{} 에 저장 — 커밋하면 다른 머신·에이전트도 이 값을 씁니다",
                    path.display()
                );
            }
            Err(e) => self.status = format!("저장 실패: {e}"),
        }
    }
}

pub fn run(config: Config, config_path: PathBuf, db_path: PathBuf) -> Result<()> {
    let mut app = App::new(config, config_path, db_path)?;

    enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

    let result = event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    std::io::stdout().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }
        let Event::Key(key) = event::read()? else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Char('q') if !app.editing => return Ok(()),
            KeyCode::Esc => {
                if app.editing {
                    app.editing = false;
                } else {
                    return Ok(());
                }
            }
            KeyCode::Tab => {
                let i = Screen::ALL.iter().position(|s| *s == app.screen).unwrap_or(0);
                app.screen = Screen::ALL[(i + 1) % Screen::ALL.len()];
                if app.screen == Screen::Impact {
                    app.run_impact();
                }
            }
            KeyCode::Enter => {
                app.editing = false;
                app.run_query();
            }
            KeyCode::Char('i') if !app.editing => app.editing = true,
            KeyCode::Char('s') if !app.editing => app.save_weights(),
            KeyCode::Backspace if app.editing => {
                app.input.pop();
            }
            KeyCode::Char(c) if app.editing => app.input.push(c),
            KeyCode::Down => match app.screen {
                Screen::Pack => {
                    app.weight_cursor = (app.weight_cursor + 1) % WEIGHT_LABELS.len();
                }
                _ => {
                    let next = app.list_state.selected().map_or(0, |i| i + 1);
                    if next < app.results.len() {
                        app.list_state.select(Some(next));
                    }
                }
            },
            KeyCode::Up => match app.screen {
                Screen::Pack => {
                    app.weight_cursor =
                        (app.weight_cursor + WEIGHT_LABELS.len() - 1) % WEIGHT_LABELS.len();
                }
                _ => {
                    let prev = app.list_state.selected().unwrap_or(0).saturating_sub(1);
                    app.list_state.select(Some(prev));
                }
            },
            // 가중치 조정 → 즉시 재랭킹. 감이 아니라 관찰로 튜닝한다.
            KeyCode::Left if app.screen == Screen::Pack => {
                app.adjust_weight(app.weight_cursor, -0.05);
                app.dirty_weights = true;
                app.rebuild_pack();
            }
            KeyCode::Right if app.screen == Screen::Pack => {
                app.adjust_weight(app.weight_cursor, 0.05);
                app.dirty_weights = true;
                app.rebuild_pack();
            }
            _ => {}
        }
    }
}

fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(3),
    ])
    .split(f.area());

    let tabs = Tabs::new(Screen::ALL.iter().map(|s| s.title()).collect::<Vec<_>>())
        .select(Screen::ALL.iter().position(|s| *s == app.screen).unwrap_or(0))
        .block(Block::default().borders(Borders::ALL).title(format!(
            " nunchi · {} ",
            app.db_path.display()
        )))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[0]);

    let prompt = if app.editing { "▸ " } else { "  " };
    let input = Paragraph::new(format!("{prompt}{}", app.input)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(if app.screen == Screen::Pack { " 태스크 " } else { " 질의 " }),
    );
    f.render_widget(input, chunks[1]);

    match app.screen {
        Screen::Pack => draw_pack(f, chunks[2], app),
        Screen::Index => draw_index(f, chunks[2], app),
        Screen::Bench => draw_bench(f, chunks[2], app),
        _ => draw_list(f, chunks[2], app),
    }

    let hint = match app.screen {
        Screen::Pack => "[←→] 가중치 · [↑↓] 선택 · [s] 공용설정 저장 · [i] 입력 · [tab] 화면 · [q] 종료",
        _ => "[↑↓] 이동 · [enter] 실행 · [i] 입력 · [tab] 화면 · [q] 종료",
    };
    let dirty = if app.dirty_weights { "  ● 저장 안 됨" } else { "" };
    let status = Paragraph::new(format!("{}{}\n{hint}", app.status, dirty))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[3]);
}

fn draw_list(f: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app
        .results
        .iter()
        .map(|r| ListItem::new(format!("{}  {}", r.label, r.reference)))
        .collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" 결과 "))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_pack(f: &mut Frame, area: Rect, app: &mut App) {
    let cols = Layout::vertical([Constraint::Min(5), Constraint::Length(7)]).split(area);

    let rows: Vec<Row> = app
        .pack
        .as_ref()
        .map(|p| {
            p.items
                .iter()
                .map(|i| {
                    Row::new(vec![
                        i.tier.to_string(),
                        i.tokens.to_string(),
                        i.sym.clone(),
                        i.reference.clone(),
                    ])
                })
                .collect()
        })
        .unwrap_or_default();

    let used = app.pack.as_ref().map(|p| p.used).unwrap_or(0);
    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Length(30),
            Constraint::Min(20),
        ],
    )
    .header(Row::new(vec!["tier", "tok", "symbol", "ref"]).style(Style::default().fg(Color::Cyan)))
    .block(Block::default().borders(Borders::ALL).title(format!(
        " 컨텍스트 팩 — used {}/{} ",
        used, app.budget
    )));
    f.render_widget(table, cols[0]);

    // 가중치 슬라이더 — 움직이면 즉시 재랭킹된다
    let inner = Layout::vertical([Constraint::Length(1); 5]).split(
        Block::default()
            .borders(Borders::ALL)
            .title(" 랭킹 가중치 (←→ 조정) ")
            .inner(cols[1]),
    );
    f.render_widget(
        Block::default().borders(Borders::ALL).title(" 랭킹 가중치 (←→ 조정) "),
        cols[1],
    );
    let values = app.weight_values();
    for (i, label) in WEIGHT_LABELS.iter().enumerate() {
        let marker = if i == app.weight_cursor { "▸" } else { " " };
        let gauge = Gauge::default()
            .ratio((values[i] / 2.0).clamp(0.0, 1.0) as f64)
            .label(format!("{marker} {label:<12} {:.2}", values[i]))
            .gauge_style(Style::default().fg(if i == app.weight_cursor {
                Color::Yellow
            } else {
                Color::DarkGray
            }));
        f.render_widget(gauge, inner[i]);
    }
}

fn draw_index(f: &mut Frame, area: Rect, app: &mut App) {
    let m = &app.metrics;
    let get = |k: &str| m.get(k).and_then(|v| v.as_u64()).unwrap_or(0);
    let mut lines = vec![
        format!("솔루션   {}", app.config.solution.name),
        format!(
            "노드 {} · 엣지 {}",
            app.store.count_nodes().unwrap_or(0),
            app.store.count_edges().unwrap_or(0)
        ),
        String::new(),
        "언어 커버리지".into(),
    ];
    if let Some(langs) = m.get("by_lang").and_then(|v| v.as_array()) {
        for e in langs {
            let files = e["files"].as_u64().unwrap_or(0);
            let parsed = e["parsed"].as_u64().unwrap_or(0);
            let pct = if files > 0 { parsed as f64 / files as f64 * 100.0 } else { 0.0 };
            lines.push(format!(
                "  {:<14}{files:>6} files  {parsed:>6} 파싱  {pct:>5.1}%",
                e["lang"].as_str().unwrap_or("?")
            ));
        }
    }
    lines.push(String::new());
    lines.push(format!(
        "라우트 {} · Bean {} · 주입 {}/{}",
        get("routes"),
        get("beans"),
        get("injects_resolved"),
        get("injects_resolved") + get("injects_unresolved")
    ));
    lines.push(format!(
        "커밋 {} · 저자 {} · 동시변경쌍 {}",
        get("commits"),
        get("authors"),
        get("cochange_pairs")
    ));
    lines.push(format!(
        "캐시 적중 {}/{}",
        get("cache_hits"),
        get("cache_hits") + get("cache_misses")
    ));

    f.render_widget(
        Paragraph::new(lines.join("\n"))
            .block(Block::default().borders(Borders::ALL).title(" 인덱스 상태 ")),
        area,
    );
}

fn draw_bench(f: &mut Frame, area: Rect, app: &mut App) {
    let m = &app.metrics;
    let get = |k: &str| m.get(k).and_then(|v| v.as_u64()).unwrap_or(0);
    let api = get("api_calls");
    let linked = get("api_calls_linked");
    let mut lines = vec![
        "교차 저장소 계약 (CALLS_API)".into(),
        format!(
            "  프런트 API 호출 {api} — 연결 {linked} ({}%)",
            if api > 0 { linked * 100 / api } else { 0 }
        ),
        format!("  동적 경로 {} (정적 분석 불가)", get("api_calls_dynamic")),
        String::new(),
        format!(
            "호출 연결률 {:.1}%",
            m.get("call_link_rate").and_then(|v| v.as_f64()).unwrap_or(0.0) * 100.0
        ),
        format!(
            "  해소 {} · 모호 {} · 미해소 {}",
            get("calls_resolved"),
            get("calls_ambiguous"),
            get("calls_unresolved")
        ),
        String::new(),
        "미해소 상위 — 외부 API면 정상, 내부 심볼이면 추출기 결함".into(),
    ];
    if let Some(top) = m.get("top_unresolved").and_then(|v| v.as_array()) {
        for e in top.iter().take(8) {
            lines.push(format!(
                "  {:<26}{:>6}",
                e["name"].as_str().unwrap_or("?"),
                e["count"].as_u64().unwrap_or(0)
            ));
        }
    }
    f.render_widget(
        Paragraph::new(lines.join("\n"))
            .block(Block::default().borders(Borders::ALL).title(" 지표 ")),
        area,
    );
}

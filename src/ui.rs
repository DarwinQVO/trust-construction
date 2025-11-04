use crate::db::Transaction;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use std::io;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    BankStatements,
    TransactionLedger,
    Views,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterType {
    None,
    AllTransactions,
    Gastos,
    Ingresos,
    PagoTarjeta,
    Traspasos,
    ByBank(String),
    ByDateRange,
    ByAmountRange,
}

#[derive(Debug, Clone)]
pub struct FilterState {
    pub active_filter: FilterType,
}

impl Page {
    pub fn next(&self) -> Self {
        match self {
            Page::BankStatements => Page::TransactionLedger,
            Page::TransactionLedger => Page::Views,
            Page::Views => Page::BankStatements,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Page::BankStatements => Page::Views,
            Page::TransactionLedger => Page::BankStatements,
            Page::Views => Page::TransactionLedger,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Page::BankStatements => "Bank Statements",
            Page::TransactionLedger => "Transaction Ledger",
            Page::Views => "Views",
        }
    }
}

pub struct App {
    pub transactions: Vec<Transaction>,
    pub filtered_transactions: Vec<Transaction>,
    pub state: TableState,
    pub total_count: i64,
    pub current_page: Page,
    pub bank_statements_state: TableState,
    pub show_detail: bool,
    pub filter_state: FilterState,
}

impl App {
    pub fn new(transactions: Vec<Transaction>, total_count: i64) -> Self {
        let mut state = TableState::default();
        if !transactions.is_empty() {
            state.select(Some(0));
        }

        let mut bank_statements_state = TableState::default();
        bank_statements_state.select(Some(0));

        let filtered_transactions = transactions.clone();

        Self {
            transactions,
            filtered_transactions,
            state,
            total_count,
            current_page: Page::TransactionLedger,
            bank_statements_state,
            show_detail: false,
            filter_state: FilterState {
                active_filter: FilterType::None,
            },
        }
    }

    pub fn toggle_detail(&mut self) {
        self.show_detail = !self.show_detail;
    }

    pub fn selected_transaction(&self) -> Option<&Transaction> {
        self.state.selected().and_then(|i| self.filtered_transactions.get(i))
    }

    pub fn apply_filter(&mut self, filter: FilterType) {
        self.filter_state.active_filter = filter.clone();

        self.filtered_transactions = match filter {
            FilterType::None | FilterType::AllTransactions => self.transactions.clone(),
            FilterType::Gastos => self.transactions.iter()
                .filter(|tx| tx.transaction_type == "GASTO")
                .cloned()
                .collect(),
            FilterType::Ingresos => self.transactions.iter()
                .filter(|tx| tx.transaction_type == "INGRESO")
                .cloned()
                .collect(),
            FilterType::PagoTarjeta => self.transactions.iter()
                .filter(|tx| tx.transaction_type == "PAGO_TARJETA")
                .cloned()
                .collect(),
            FilterType::Traspasos => self.transactions.iter()
                .filter(|tx| tx.transaction_type == "TRASPASO")
                .cloned()
                .collect(),
            FilterType::ByBank(ref bank) => self.transactions.iter()
                .filter(|tx| &tx.bank == bank)
                .cloned()
                .collect(),
            FilterType::ByDateRange | FilterType::ByAmountRange => {
                // Placeholder for future implementation
                self.transactions.clone()
            }
        };

        // Reset selection to first item
        if !self.filtered_transactions.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }

    pub fn clear_filter(&mut self) {
        self.apply_filter(FilterType::None);
    }

    pub fn next_page(&mut self) {
        self.current_page = self.current_page.next();
    }

    pub fn previous_page(&mut self) {
        self.current_page = self.current_page.previous();
    }

    pub fn bank_summary(&self) -> Vec<(String, usize, f64)> {
        let mut summary: HashMap<String, (usize, f64)> = HashMap::new();

        for tx in &self.transactions {
            let entry = summary.entry(tx.bank.clone()).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += tx.amount_numeric;
        }

        let mut result: Vec<_> = summary
            .into_iter()
            .map(|(bank, (count, total))| (bank, count, total))
            .collect();

        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    pub fn next(&mut self) {
        let len = self.filtered_transactions.len();
        if len == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let len = self.filtered_transactions.len();
        if len == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn page_down(&mut self) {
        let len = self.filtered_transactions.len();
        if len == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                let next = i + 20;
                if next >= len {
                    len - 1
                } else {
                    next
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn page_up(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i < 20 {
                    0
                } else {
                    i - 20
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn stats(&self) -> TransactionStats {
        let mut stats = TransactionStats::default();

        for tx in &self.transactions {
            match tx.transaction_type.as_str() {
                "GASTO" => {
                    stats.gastos_count += 1;
                    stats.gastos_total += tx.amount_numeric;
                }
                "INGRESO" => {
                    stats.ingresos_count += 1;
                    stats.ingresos_total += tx.amount_numeric;
                }
                "PAGO_TARJETA" => stats.pago_tarjeta_count += 1,
                "TRASPASO" => stats.traspaso_count += 1,
                _ => {}
            }
        }

        stats
    }
}

#[derive(Default)]
pub struct TransactionStats {
    pub gastos_count: usize,
    pub gastos_total: f64,
    pub ingresos_count: usize,
    pub ingresos_total: f64,
    pub pago_tarjeta_count: usize,
    pub traspaso_count: usize,
}

pub fn run_ui(app: &mut App) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Enter => app.toggle_detail(),
                KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        app.previous_page();
                    } else {
                        app.next_page();
                    }
                }
                KeyCode::Char('c') => {
                    app.clear_filter();
                    app.current_page = Page::TransactionLedger;
                }
                KeyCode::Char('1') if app.current_page == Page::Views => {
                    app.apply_filter(FilterType::AllTransactions);
                    app.current_page = Page::TransactionLedger;
                }
                KeyCode::Char('2') if app.current_page == Page::Views => {
                    app.apply_filter(FilterType::Gastos);
                    app.current_page = Page::TransactionLedger;
                }
                KeyCode::Char('3') if app.current_page == Page::Views => {
                    app.apply_filter(FilterType::Ingresos);
                    app.current_page = Page::TransactionLedger;
                }
                KeyCode::Char('4') if app.current_page == Page::Views => {
                    app.apply_filter(FilterType::PagoTarjeta);
                    app.current_page = Page::TransactionLedger;
                }
                KeyCode::Char('5') if app.current_page == Page::Views => {
                    app.apply_filter(FilterType::Traspasos);
                    app.current_page = Page::TransactionLedger;
                }
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                KeyCode::PageDown => app.page_down(),
                KeyCode::PageUp => app.page_up(),
                KeyCode::Home => app.state.select(Some(0)),
                KeyCode::End => {
                    if !app.filtered_transactions.is_empty() {
                        app.state.select(Some(app.filtered_transactions.len() - 1));
                    }
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with navigation
            Constraint::Min(0),    // Content area
            Constraint::Length(3), // Status bar
        ])
        .split(f.size());

    // Header with page navigation
    render_header(f, chunks[0], app);

    // Content area with optional split for detail panel
    if app.show_detail && app.current_page == Page::TransactionLedger {
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Transaction list
                Constraint::Percentage(40), // Detail panel
            ])
            .split(chunks[1]);

        render_table(f, content_chunks[0], app);
        render_detail_panel(f, content_chunks[1], app);
    } else {
        // Normal full-width content
        match app.current_page {
            Page::BankStatements => render_bank_statements(f, chunks[1], app),
            Page::TransactionLedger => render_table(f, chunks[1], app),
            Page::Views => render_views(f, chunks[1], app),
        }
    }

    // Status bar
    render_status_bar(f, chunks[2], app);
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let stats = app.stats();

    // Page tabs
    let pages = vec![
        (Page::BankStatements, "Bank Statements"),
        (Page::TransactionLedger, "Transaction Ledger"),
        (Page::Views, "Views"),
    ];

    let mut tab_spans = vec![];
    for (i, (page, name)) in pages.iter().enumerate() {
        if i > 0 {
            tab_spans.push(Span::raw(" │ "));
        }

        let style = if *page == app.current_page {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        tab_spans.push(Span::styled(*name, style));
    }

    tab_spans.push(Span::raw("  |  "));
    tab_spans.push(Span::styled(
        format!("Total: {}", app.total_count),
        Style::default().fg(Color::White),
    ));
    tab_spans.push(Span::raw("  |  "));
    tab_spans.push(Span::styled(
        format!("↓ {}", stats.gastos_count),
        Style::default().fg(Color::Red),
    ));
    tab_spans.push(Span::raw("  "));
    tab_spans.push(Span::styled(
        format!("↑ {}", stats.ingresos_count),
        Style::default().fg(Color::Green),
    ));

    let header_text = vec![Line::from(tab_spans)];

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));

    f.render_widget(header, area);
}

fn render_table(f: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Date", "Bank", "Merchant", "Amount", "Type", "Category"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });

    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::DarkGray))
        .height(1);

    let rows = app.filtered_transactions.iter().map(|tx| {
        let color = match tx.transaction_type.as_str() {
            "GASTO" => Color::Red,
            "INGRESO" => Color::Green,
            "PAGO_TARJETA" => Color::Yellow,
            "TRASPASO" => Color::Cyan,
            _ => Color::White,
        };

        let cells = vec![
            Cell::from(tx.date.clone()),
            Cell::from(tx.bank.clone()),
            Cell::from(truncate(&tx.merchant, 30)),
            Cell::from(format!("{:.2}", tx.amount_numeric)).style(Style::default().fg(color)),
            Cell::from(tx.transaction_type.clone()).style(Style::default().fg(color)),
            Cell::from(truncate(&tx.category, 20)),
        ];

        Row::new(cells).height(1)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(18),
            Constraint::Length(32),
            Constraint::Length(12),
            Constraint::Length(15),
            Constraint::Length(22),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .title(" Transactions "),
    )
    .highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("→ ");

    f.render_stateful_widget(table, area, &mut app.state);
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let selected = app.state.selected().map(|i| i + 1).unwrap_or(0);
    let total = app.filtered_transactions.len();

    let mut status_spans = vec![
        Span::styled(
            format!(" Row: {}/{} ", selected, total),
            Style::default().fg(Color::Cyan),
        ),
    ];

    // Show filter status if active
    if app.filter_state.active_filter != FilterType::None
        && app.filter_state.active_filter != FilterType::AllTransactions {
        let filter_name = match &app.filter_state.active_filter {
            FilterType::Gastos => "GASTO",
            FilterType::Ingresos => "INGRESO",
            FilterType::PagoTarjeta => "PAGO_TARJETA",
            FilterType::Traspasos => "TRASPASO",
            FilterType::ByBank(bank) => bank.as_str(),
            _ => "CUSTOM",
        };
        status_spans.push(Span::raw(" | "));
        status_spans.push(Span::styled(
            format!("Filter: {}", filter_name),
            Style::default().fg(Color::Green),
        ));
        status_spans.push(Span::raw(" ("));
        status_spans.push(Span::styled("c", Style::default().fg(Color::Yellow)));
        status_spans.push(Span::raw(" clear)"));
    }

    status_spans.push(Span::raw(" | "));
    status_spans.push(Span::styled("Enter", Style::default().fg(Color::Yellow)));
    status_spans.push(Span::raw(" Details | "));
    status_spans.push(Span::styled("Tab", Style::default().fg(Color::Yellow)));
    status_spans.push(Span::raw(" Page | "));
    status_spans.push(Span::styled("↑/↓", Style::default().fg(Color::Yellow)));
    status_spans.push(Span::raw(" Nav | "));
    status_spans.push(Span::styled("PgUp/PgDn", Style::default().fg(Color::Yellow)));
    status_spans.push(Span::raw(" Fast | "));
    status_spans.push(Span::styled("q", Style::default().fg(Color::Red)));
    status_spans.push(Span::raw(" Quit"));

    let status_text = vec![Line::from(status_spans)];

    let status_bar = Paragraph::new(status_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)),
    );

    f.render_widget(status_bar, area);
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn render_bank_statements(f: &mut Frame, area: Rect, app: &mut App) {
    let bank_summary = app.bank_summary();

    let header_cells = ["Bank", "Transactions", "Total Amount", "Avg Amount"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });

    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::DarkGray))
        .height(1);

    let rows = bank_summary.iter().map(|(bank, count, total)| {
        let avg = total / *count as f64;
        let color = if *total > 0.0 {
            Color::Green
        } else {
            Color::Red
        };

        let cells = vec![
            Cell::from(bank.clone()),
            Cell::from(format!("{}", count)),
            Cell::from(format!("{:.2}", total)).style(Style::default().fg(color)),
            Cell::from(format!("{:.2}", avg)),
        ];

        Row::new(cells).height(1)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(25),
            Constraint::Length(15),
            Constraint::Length(18),
            Constraint::Length(18),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .title(" Bank Statements - Summary by Bank "),
    )
    .highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("→ ");

    f.render_stateful_widget(table, area, &mut app.bank_statements_state);
}

fn render_views(f: &mut Frame, area: Rect, app: &App) {
    let stats = app.stats();

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Quick Views & Filters",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from("  ╔══════════════════════════════════════════════════╗"),
        Line::from(vec![
            Span::raw("  ║ "),
            if app.filter_state.active_filter == FilterType::AllTransactions {
                Span::styled("→", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            } else {
                Span::raw(" ")
            },
            Span::styled("1", Style::default().fg(Color::Yellow)),
            Span::raw(". All Transactions          "),
            Span::styled(
                format!("{:>5} txs", app.total_count),
                Style::default().fg(Color::White),
            ),
            Span::raw("         ║"),
        ]),
        Line::from("  ╠══════════════════════════════════════════════════╣"),
        Line::from(vec![
            Span::raw("  ║ "),
            if app.filter_state.active_filter == FilterType::Gastos {
                Span::styled("→", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            } else {
                Span::raw(" ")
            },
            Span::styled("2", Style::default().fg(Color::Yellow)),
            Span::raw(". Expenses (GASTO)          "),
            Span::styled(
                format!("{:>5} txs", stats.gastos_count),
                Style::default().fg(Color::Red),
            ),
            Span::raw("         ║"),
        ]),
        Line::from(vec![
            Span::raw("  ║ "),
            if app.filter_state.active_filter == FilterType::Ingresos {
                Span::styled("→", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            } else {
                Span::raw(" ")
            },
            Span::styled("3", Style::default().fg(Color::Yellow)),
            Span::raw(". Income (INGRESO)          "),
            Span::styled(
                format!("{:>5} txs", stats.ingresos_count),
                Style::default().fg(Color::Green),
            ),
            Span::raw("         ║"),
        ]),
        Line::from(vec![
            Span::raw("  ║ "),
            if app.filter_state.active_filter == FilterType::PagoTarjeta {
                Span::styled("→", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            } else {
                Span::raw(" ")
            },
            Span::styled("4", Style::default().fg(Color::Yellow)),
            Span::raw(". Credit Card Payments      "),
            Span::styled(
                format!("{:>5} txs", stats.pago_tarjeta_count),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw("         ║"),
        ]),
        Line::from(vec![
            Span::raw("  ║ "),
            if app.filter_state.active_filter == FilterType::Traspasos {
                Span::styled("→", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            } else {
                Span::raw(" ")
            },
            Span::styled("5", Style::default().fg(Color::Yellow)),
            Span::raw(". Transfers (TRASPASO)      "),
            Span::styled(
                format!("{:>5} txs", stats.traspaso_count),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("         ║"),
        ]),
        Line::from("  ╠══════════════════════════════════════════════════╣"),
        Line::from(vec![
            Span::raw("  ║ "),
            Span::styled("6", Style::default().fg(Color::Yellow)),
            Span::raw(". By Bank...                "),
            Span::styled("5 banks", Style::default().fg(Color::White)),
            Span::raw("          ║"),
        ]),
        Line::from(vec![
            Span::raw("  ║ "),
            Span::styled("7", Style::default().fg(Color::Yellow)),
            Span::raw(". By Date Range...          "),
            Span::styled("Custom", Style::default().fg(Color::White)),
            Span::raw("          ║"),
        ]),
        Line::from(vec![
            Span::raw("  ║ "),
            Span::styled("8", Style::default().fg(Color::Yellow)),
            Span::raw(". By Amount Range...        "),
            Span::styled("Custom", Style::default().fg(Color::White)),
            Span::raw("          ║"),
        ]),
        Line::from("  ╚══════════════════════════════════════════════════╝"),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Hint: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            ),
            Span::styled(
                "Press ",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ),
            Span::styled(
                "1-5",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            ),
            Span::styled(
                " to filter, ",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ),
            Span::styled(
                "c",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            ),
            Span::styled(
                " to clear",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .title(" Views - Quick Access Filters "),
    );

    f.render_widget(paragraph, area);
}

fn render_detail_panel(f: &mut Frame, area: Rect, app: &App) {
    let tx = match app.selected_transaction() {
        Some(t) => t,
        None => {
            let no_selection = Paragraph::new("No transaction selected")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow))
                        .title(" Transaction Details "),
                );
            f.render_widget(no_selection, area);
            return;
        }
    };

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Date: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(&tx.date),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Merchant: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(&tx.merchant),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Amount: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:.2}", tx.amount_numeric),
                Style::default().fg(if tx.amount_numeric < 0.0 { Color::Red } else { Color::Green }),
            ),
            Span::raw(" "),
            Span::raw(&tx.currency),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Type: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(
                &tx.transaction_type,
                Style::default().fg(match tx.transaction_type.as_str() {
                    "GASTO" => Color::Red,
                    "INGRESO" => Color::Green,
                    "PAGO_TARJETA" => Color::Yellow,
                    "TRASPASO" => Color::Cyan,
                    _ => Color::White,
                }),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Category: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(&tx.category),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Bank: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(&tx.bank),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Account: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(&tx.account_name),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Account #: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(&tx.account_number),
        ]),
        Line::from(""),
        Line::from("  ─────────────────────────────────────"),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  PROVENANCE",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Source File: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(&tx.source_file, Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Line Number: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(&tx.line_number, Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from("  ─────────────────────────────────────"),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  DESCRIPTION",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                wrap_text(&tx.description, 35),
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Press Enter to close",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]),
    ];

    let detail_panel = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" Transaction Details "),
    );

    f.render_widget(detail_panel, area);
}

fn wrap_text(text: &str, width: usize) -> String {
    if text.len() <= width {
        text.to_string()
    } else {
        let mut result = String::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_line = String::new();

        for word in words {
            if current_line.len() + word.len() + 1 <= width {
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
            } else {
                if !result.is_empty() {
                    result.push_str("\n  ");
                }
                result.push_str(&current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            if !result.is_empty() {
                result.push_str("\n  ");
            }
            result.push_str(&current_line);
        }

        result
    }
}

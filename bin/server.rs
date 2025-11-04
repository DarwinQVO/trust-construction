// Trust Construction System - Web Server
// Badge 13: REST API with Axum

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use trust_construction::{get_all_transactions, get_source_file_stats, get_transactions_by_source, Transaction, SourceFileStat};

/// Shared application state
#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Connection>>,
}

/// API Response wrapper
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self {
            success: true,
            data,
            error: None,
        }
    }
}

/// Stats response
#[derive(Serialize)]
struct StatsResponse {
    total_transactions: usize,
    total_expenses: f64,
    total_income: f64,
    total_transfers: f64,
    total_credit_payments: f64,
    by_bank: Vec<BankStat>,
}

#[derive(Serialize)]
struct BankStat {
    bank: String,
    count: usize,
    total: f64,
}

/// Transaction response (simplified for API)
#[derive(Serialize, Deserialize)]
struct TransactionResponse {
    date: String,
    description: String,
    amount_numeric: f64,
    transaction_type: String,
    category: String,
    merchant: String,
    bank: String,
    source_file: String,
}

/// Source file response
#[derive(Serialize)]
struct SourceFileResponse {
    source_file: String,
    bank: String,
    transaction_count: i64,
    total_expenses: f64,
    total_income: f64,
    date_range: String,
}

impl From<Transaction> for TransactionResponse {
    fn from(tx: Transaction) -> Self {
        Self {
            date: tx.date,
            description: tx.description,
            amount_numeric: tx.amount_numeric,
            transaction_type: tx.transaction_type,
            category: tx.category,
            merchant: tx.merchant,
            bank: tx.bank,
            source_file: tx.source_file,
        }
    }
}

impl From<SourceFileStat> for SourceFileResponse {
    fn from(stat: SourceFileStat) -> Self {
        Self {
            source_file: stat.source_file,
            bank: stat.bank,
            transaction_count: stat.transaction_count,
            total_expenses: stat.total_expenses,
            total_income: stat.total_income,
            date_range: stat.date_range,
        }
    }
}

// ============================================================================
// API Handlers
// ============================================================================

/// GET /api/health - Health check
async fn health_check() -> impl IntoResponse {
    Json(ApiResponse::ok("OK"))
}

/// GET /api/transactions - Get all transactions
async fn get_transactions(State(state): State<AppState>) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();

    match get_all_transactions(&conn) {
        Ok(transactions) => {
            let response: Vec<TransactionResponse> = transactions
                .into_iter()
                .map(|tx| tx.into())
                .collect();

            (StatusCode::OK, Json(ApiResponse::ok(response))).into_response()
        }
        Err(e) => {
            eprintln!("Error getting transactions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::ok(Vec::<TransactionResponse>::new())),
            )
                .into_response()
        }
    }
}

/// GET /api/stats - Get statistics
async fn get_stats(State(state): State<AppState>) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();

    match get_all_transactions(&conn) {
        Ok(transactions) => {
            let total = transactions.len();

            let mut total_expenses = 0.0;
            let mut total_income = 0.0;
            let mut total_transfers = 0.0;
            let mut total_credit_payments = 0.0;

            let mut bank_stats: std::collections::HashMap<String, (usize, f64)> =
                std::collections::HashMap::new();

            for tx in &transactions {
                // Update totals by type
                match tx.transaction_type.as_str() {
                    "GASTO" => total_expenses += tx.amount_numeric.abs(),
                    "INGRESO" => total_income += tx.amount_numeric.abs(),
                    "TRASPASO" => total_transfers += tx.amount_numeric.abs(),
                    "PAGO_TARJETA" => total_credit_payments += tx.amount_numeric.abs(),
                    _ => {}
                }

                // Update bank stats
                let entry = bank_stats.entry(tx.bank.clone()).or_insert((0, 0.0));
                entry.0 += 1;
                entry.1 += tx.amount_numeric.abs();
            }

            let by_bank: Vec<BankStat> = bank_stats
                .into_iter()
                .map(|(bank, (count, total))| BankStat { bank, count, total })
                .collect();

            let stats = StatsResponse {
                total_transactions: total,
                total_expenses,
                total_income,
                total_transfers,
                total_credit_payments,
                by_bank,
            };

            (StatusCode::OK, Json(ApiResponse::ok(stats))).into_response()
        }
        Err(e) => {
            eprintln!("Error getting stats: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::ok(StatsResponse {
                    total_transactions: 0,
                    total_expenses: 0.0,
                    total_income: 0.0,
                    total_transfers: 0.0,
                    total_credit_payments: 0.0,
                    by_bank: vec![],
                })),
            )
                .into_response()
        }
    }
}

/// GET /api/filters/:type - Filter transactions by type
async fn filter_transactions(
    State(state): State<AppState>,
    Path(filter_type): Path<String>,
) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();

    match get_all_transactions(&conn) {
        Ok(transactions) => {
            let filtered: Vec<TransactionResponse> = transactions
                .into_iter()
                .filter(|tx| {
                    filter_type == "all" || tx.transaction_type.to_lowercase() == filter_type.to_lowercase()
                })
                .map(|tx| tx.into())
                .collect();

            (StatusCode::OK, Json(ApiResponse::ok(filtered))).into_response()
        }
        Err(e) => {
            eprintln!("Error filtering transactions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::ok(Vec::<TransactionResponse>::new())),
            )
                .into_response()
        }
    }
}

/// GET /api/sources - Get all source files with statistics
async fn get_sources(State(state): State<AppState>) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();

    match get_source_file_stats(&conn) {
        Ok(stats) => {
            let response: Vec<SourceFileResponse> = stats
                .into_iter()
                .map(|stat| stat.into())
                .collect();

            (StatusCode::OK, Json(ApiResponse::ok(response))).into_response()
        }
        Err(e) => {
            eprintln!("Error getting source files: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::ok(Vec::<SourceFileResponse>::new())),
            )
                .into_response()
        }
    }
}

/// GET /api/sources/:filename - Get transactions from a specific source file
async fn get_source_transactions(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();

    // Decode URL-encoded filename
    let decoded_filename = urlencoding::decode(&filename)
        .unwrap_or_else(|_| filename.clone().into())
        .into_owned();

    match get_transactions_by_source(&conn, &decoded_filename) {
        Ok(transactions) => {
            let response: Vec<TransactionResponse> = transactions
                .into_iter()
                .map(|tx| tx.into())
                .collect();

            (StatusCode::OK, Json(ApiResponse::ok(response))).into_response()
        }
        Err(e) => {
            eprintln!("Error getting transactions for source {}: {}", decoded_filename, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::ok(Vec::<TransactionResponse>::new())),
            )
                .into_response()
        }
    }
}

/// GET / - Serve index.html
async fn serve_index() -> impl IntoResponse {
    Html(include_str!("../web/index.html"))
}

/// GET /statements - Serve statements page
async fn serve_statements() -> impl IntoResponse {
    Html(include_str!("../web/statements.html"))
}

/// GET /statement-detail - Serve statement detail page
async fn serve_statement_detail() -> impl IntoResponse {
    Html(include_str!("../web/statement-detail.html"))
}

// ============================================================================
// Main Server
// ============================================================================

#[tokio::main]
async fn main() {
    println!("🌐 Trust Construction System - Web Server");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Open database
    let db_path = std::path::Path::new("/Users/darwinborges/finance/trust-construction/transactions.db");

    if !db_path.exists() {
        eprintln!("❌ Database not found at {:?}", db_path);
        eprintln!("   Run: cargo run --release import");
        eprintln!("   to import transactions first.");
        std::process::exit(1);
    }

    let conn = Connection::open(db_path).expect("Failed to open database");
    println!("✓ Database opened: {:?}", db_path);

    // Create shared state
    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
    };

    // Build API routes
    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/transactions", get(get_transactions))
        .route("/stats", get(get_stats))
        .route("/filters/:type", get(filter_transactions))
        .route("/sources", get(get_sources))
        .route("/sources/:filename", get(get_source_transactions))
        .with_state(state.clone());

    // Build main router
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/statements", get(serve_statements))
        .route("/statement-detail", get(serve_statement_detail))
        .nest("/api", api_routes)
        .nest_service("/static", ServeDir::new("web"))
        .layer(CorsLayer::permissive());

    // Start server
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    println!("\n🚀 Server running on http://localhost:3000");
    println!("   API: http://localhost:3000/api/transactions");
    println!("   UI:  http://localhost:3000");
    println!("\n   Press Ctrl+C to stop\n");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

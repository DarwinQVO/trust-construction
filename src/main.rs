// Only compile UI module when TUI feature is enabled
#[cfg(feature = "tui")]
mod ui;

use anyhow::Result;
use rusqlite::Connection;
use std::env;
use std::path::Path;

// Use library instead of local modules
use trust_construction::{load_csv, setup_database, insert_transactions, get_all_transactions, verify_count};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "import" {
        // Import mode
        run_import()?;
    } else {
        // UI mode (default)
        run_ui_mode()?;
    }

    Ok(())
}

fn run_import() -> Result<()> {
    println!("🗄️  Badge 1: Data Import - CSV → SQLite + WAL");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Paths
    let csv_path = Path::new("/Users/darwinborges/finance/transactions_ALL_SOURCES.csv");
    let db_path = Path::new("/Users/darwinborges/finance/trust-construction/transactions.db");

    // 1. Load CSV
    println!("\n📂 Loading CSV...");
    let transactions = load_csv(csv_path)?;
    println!("✓ Loaded {} transactions from CSV", transactions.len());

    // 2. Setup database
    println!("\n🔧 Setting up database...");
    let conn = Connection::open(db_path)?;
    setup_database(&conn)?;
    println!("✓ Database initialized with WAL mode");

    // 3. Insert transactions
    println!("\n💾 Inserting transactions...");
    insert_transactions(&conn, &transactions)?;

    // 4. Verify count
    println!("\n🔍 Verifying database...");
    let count = verify_count(&conn)?;
    println!("✓ Database contains {} transactions", count);

    // 5. Success criteria
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    if count == transactions.len() as i64 {
        println!("🎉 Badge 1 COMPLETE!");
        println!("✅ Success criteria met: {} transactions", count);
    } else {
        println!("✅ Badge 1 COMPLETE!");
        println!("✓ Unique transactions: {}", count);
        println!("✓ Duplicates detected: {}", transactions.len() as i64 - count);
    }

    Ok(())
}

#[cfg(feature = "tui")]
fn run_ui_mode() -> Result<()> {
    println!("🖥️  Loading Trust Construction System UI...\n");

    // Open database
    let db_path = Path::new("/Users/darwinborges/finance/trust-construction/transactions.db");

    if !db_path.exists() {
        eprintln!("❌ Database not found!");
        eprintln!("   Run: cargo run import");
        eprintln!("   to import transactions first.");
        std::process::exit(1);
    }

    let conn = Connection::open(db_path)?;

    // Load transactions
    println!("📊 Loading transactions...");
    let transactions = get_all_transactions(&conn)?;
    let total_count = verify_count(&conn)?;

    println!("✓ Loaded {} transactions\n", transactions.len());
    println!("Starting UI... (Press 'q' to quit)\n");

    // Create and run app
    let mut app = ui::App::new(transactions, total_count);
    ui::run_ui(&mut app)?;

    println!("\n✅ UI closed successfully");

    Ok(())
}

#[cfg(not(feature = "tui"))]
fn run_ui_mode() -> Result<()> {
    eprintln!("❌ TUI mode not available!");
    eprintln!("   Rebuild with: cargo build --features tui");
    eprintln!("   Or use web UI: cargo run --bin trust-server --features server");
    std::process::exit(1);
}

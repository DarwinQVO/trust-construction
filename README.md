# 🗄️ Trust Construction System

> Converting untrustworthy financial data into trustworthy data with mathematical guarantees.

**Status:** 11/20 badges complete (55%) - Tier 1 COMPLETE! Tier 2: 6/10 🔐

---

## 🚀 Quick Start

### Launch the UI

```bash
cargo run --release
```

### Import transactions (first time)

```bash
cargo run --release import
```

Or use the convenience script:

```bash
./run.sh ui      # Launch UI
./run.sh import  # Import data
```

---

## 📊 Current Features

### ✅ Badge 1: Data Import
- **4,512** unique transactions imported
- **365** duplicates detected automatically
- SHA-256 idempotency hash
- SQLite with WAL mode
- Complete provenance tracking

### ✅ Badge 2: Terminal UI
- Interactive table with 4,512 transactions
- Color-coded by type:
  - 🔴 Red = GASTO (expenses)
  - 🟢 Green = INGRESO (income)
  - 🟡 Yellow = PAGO_TARJETA (credit card payment)
  - 🔵 Cyan = TRASPASO (transfer)
- Keyboard navigation: ↑/↓, PgUp/PgDn, Home/End
- Live statistics in header
- Status bar with shortcuts

### ✅ Badge 3: Multi-Page Navigation
- **3 pages:** Bank Statements, Transaction Ledger, Views
- Tab key switching (Tab / Shift+Tab)
- **Bank Statements:** Summary by bank with totals
  - 5 banks: BofA (3055), Scotia (895), Apple (488), Wise (44), Stripe (30)
  - Shows: count, total amount, average per transaction
- **Transaction Ledger:** Full list (original view from Badge 2)
- **Views:** Preview of 8 quick filters (implementation in Badge 5)
- Visual feedback: Active page highlighted in yellow + underlined
- Smooth page transitions

### ✅ Badge 4: Detail View
- **Press Enter** to toggle detail panel (60/40 split)
- Shows all 14 transaction fields in dedicated panel
- **Provenance tracking visible:**
  - Source file name
  - Line number in original file
- Full description with automatic text wrapping
- Real-time updates when navigating
- Yellow border indicates focused detail panel
- Smart toggle: closes on page change

### ✅ Badge 5: Filters
- **Real-time filtering** by transaction type
- **5 quick filters:**
  - All Transactions (4,512)
  - GASTO / Expenses (3,829) 🔴
  - INGRESO / Income (421) 🟢
  - PAGO_TARJETA / Credit Card Payments (150) 🟡
  - TRASPASO / Transfers (112) 🔵
- **Press 1-5** in Views page to filter instantly
- **Press c** anywhere to clear filter
- Status bar shows active filter + count
- Views page highlights active filter with →
- Navigation adapts to filtered list
- Memory-based filtering (<1ms)

### ✅ Badge 6: Parser Framework (Tier 2 begins!)
- **Expression Problem SOLVED** ✅
  - Add TYPES (banks): Implement trait → No code changes ✅
  - Add FUNCTIONS: Create new trait → Existing parsers untouched ✅
- **Composable traits architecture:**
  - `BankParser` (core, required)
  - `MerchantExtractor` (optional)
  - `TypeClassifier` (optional)
  - Future: `AmountValidator`, `DateNormalizer`, etc.
- **5 bank support:**
  - Bank of America, AppleCard, Stripe, Wise, Scotiabank
- **Auto-detection** - `detect_source()` identifies bank from filename
- **Factory pattern** - `get_parser()` creates appropriate parser
- **RawTransaction** struct for parser output
- 12 unit tests (100% pass rate)
- Ready for Badge 7-11 parser implementations

### ✅ Badge 7: BofA Parser
- **CSV parsing** with 3 columns: Date, Description, Amount
- **Merchant extraction** with pattern matching
  - "Stripe, Des:transfer..." → "Stripe"
  - Handles comma-separated format
- **Type classification** with 4 types:
  - PAGO_TARJETA (credit card payments)
  - TRASPASO (transfers)
  - INGRESO (income)
  - GASTO (expenses)
- Order-sensitive logic (most specific first)
- 17 total tests (5 new BofA tests)
- Test CSV with 3 real transactions

### ✅ Badge 8: AppleCard Parser
- **CSV parsing** with 5 columns: Date, Description, Amount, Category, Merchant
- **Merchant comes pre-cleaned** (unlike BofA)
- **Category included** in CSV
- **Merchant extraction** from description (first 2 words)
- **Type classification** with 2 types:
  - PAGO_TARJETA (ACH deposits - payments to card)
  - GASTO (all purchases - simpler than BofA)
- 21 total tests (4 new AppleCard tests)
- Test CSV with 3 real transactions
- Expression Problem still SOLVED ✅

### ✅ Badge 9: Stripe Parser
- **JSON parsing** with Stripe API format: `{ "data": [...] }`
- **Amount conversion** cents → dollars (286770 → $2,867.70)
- **Date conversion** Unix timestamp → MM/DD/YYYY (1735084800 → "12/25/2024")
- Uses **serde_json** for JSON parsing (first non-CSV parser!)
- **Merchant extraction** with pattern matching:
  - "Payment from X" → X
  - "Payment to Y" → Y
- **Type classification** with 2 types:
  - INGRESO (default - payouts)
  - GASTO (refunds, fees, charges)
- 26 total tests (5 new Stripe tests)
- Test JSON with 3 balance_transactions
- Expression Problem still SOLVED - JSON and CSV use same trait interface ✅

### ✅ Badge 10: Wise Parser 🎉 TIER 2 HALFWAY!
- **CSV parsing** with 9 columns (most complex CSV format so far)
- **Multi-currency support** - USD, EUR, MXN with automatic conversion ✅
- **Exchange rate conversion:**
  - 500 EUR @ rate 0.93 → $537.63 USD
  - 41,000 MXN @ rate 20.00 → $2,050.00 USD
- **Currency tracking** in description: "500 EUR → $537.63 USD @ rate 0.9300"
- Uses exchange rates from CSV for accurate conversion
- **Merchant extraction** uses Payee Name column (more reliable than pattern matching)
- **Type classification** with 3 types:
  - INGRESO (incoming payments)
  - GASTO (outgoing payments)
  - TRASPASO (currency conversions - default for Wise)
- 33 total tests (8 new Wise tests including currency conversion tests)
- Test CSV with 5 multi-currency transactions
- First parser with multi-currency support ✅
- Expression Problem still SOLVED - multi-currency uses same trait interface ✅

### ⬜ Badge 11: Scotiabank PDF Parser (OPTIONAL - SKIPPED)

### ✅ Badge 12: Idempotency 🔐
- **Already implemented in Badge 1!** - Discovered complete implementation ✅
- **SHA-256 hashing:** `compute_idempotency_hash()` using date + amount + merchant + bank
- **UNIQUE constraint** on `idempotency_hash` column prevents SQL-level duplicates
- **Duplicate detection** in `insert_transactions()` catches ConstraintViolation errors
- **Test verification:** Import same CSV twice → 0 duplicates inserted ✅
  - First import: "✓ Inserted: 3 / ✓ Skipped duplicates: 0"
  - Second import: "✓ Inserted: 0 / ✓ Skipped duplicates: 3"
- 35 total tests (2 new idempotency tests)
- **Safe re-imports:** Can re-run imports without creating duplicates
- **Performance:** O(log n) duplicate check, ~160 bytes overhead per transaction
- **Benefits:**
  - Network retry safety (crash during import → restart safely)
  - Partial file imports (import incomplete → complete file won't duplicate)
  - Cross-source duplicate prevention
- Hash determinism verified with dedicated test

---

## 🎯 What is Trust Construction?

**Trust Construction** converts unstructured data → structured data with guarantees:

```
Untrustworthy Data (CSV/PDF)
    ↓
  [Parser]
    ↓
  [Normalizer]
    ↓
  [Validator]
    ↓
Trustworthy Data (SQLite)
```

**Key features:**
- **Provenance:** Every transaction traceable to source file + line
- **Idempotency:** Same input → same output (SHA-256 hash)
- **Immutability:** SQLite WAL mode for ACID compliance
- **Validation:** Data quality checks at every step

---

## 🎨 UI Preview

```
╔═══════════════════════════════════════════════════════════════════════════╗
║ 🗄️  TRUST CONSTRUCTION  |  Total: 4512  |  ↓ Gastos: 3829  |  ↑ Ing: 421 ║
╠═══════════════════════════════════════════════════════════════════════════╣
║ Date       │ Bank        │ Merchant           │ Amount  │ Type    │ Cat.. ║
║────────────┼─────────────┼────────────────────┼─────────┼─────────┼──────║
║→10/30/2024 │ AppleCard   │ Uber Eats          │ 3.74    │ GASTO   │ Food ║
║ 10/26/2024 │ AppleCard   │ Uber* Eats         │ 71.81   │ GASTO   │ Food ║
║ [... 4,510 more rows ...]                                                 ║
╠═══════════════════════════════════════════════════════════════════════════╣
║ Row: 1/4512 | ↑/↓ Navigate | PgUp/PgDn Fast | Home/End Jump | q Quit    ║
╚═══════════════════════════════════════════════════════════════════════════╝
```

---

## 🏗️ Architecture

### Stack
- **Rust** - Memory-safe, deterministic execution
- **SQLite** - ACID compliance with WAL mode
- **Ratatui** - Terminal UI framework
- **Crossterm** - Cross-platform terminal backend

### Modules
```
src/
├── main.rs      # Entry point + routing
├── db.rs        # Database layer (232 lines)
└── ui.rs        # Terminal UI (318 lines)
```

### Data Flow
```
CSV File
  ↓
db::load_csv()
  ↓
db::insert_transactions()  (with idempotency)
  ↓
SQLite Database (WAL mode)
  ↓
db::get_all_transactions()
  ↓
ui::App
  ↓
Ratatui Terminal UI
```

---

## 📦 Data Sources

Currently processing transactions from:
- **Bank of America** (3,055 transactions - 67.7%)
- **Scotiabank** (895 transactions - 19.8%)
- **AppleCard** (488 transactions - 10.8%)
- **Wise** (44 transactions - 1.0%)
- **Stripe** (30 transactions - 0.7%)

**Total:** 4,512 unique transactions

---

## ⌨️ Keyboard Shortcuts

| Key           | Action                         | Context    |
|---------------|--------------------------------|------------|
| **Tab**       | Next page                      | Any page   |
| **Shift+Tab** | Previous page                  | Any page   |
| **Enter**     | Toggle detail panel            | Ledger     |
| **1**         | Show all transactions          | Views      |
| **2**         | Filter to GASTO (expenses)     | Views      |
| **3**         | Filter to INGRESO (income)     | Views      |
| **4**         | Filter to PAGO_TARJETA         | Views      |
| **5**         | Filter to TRASPASO (transfers) | Views      |
| **c**         | Clear filter                   | Any page   |
| ↑ / k         | Move up one row                | Any page   |
| ↓ / j         | Move down one row              | Any page   |
| PgUp          | Scroll up 20 rows              | Any page   |
| PgDn          | Scroll down 20 rows            | Any page   |
| Home          | Jump to first transaction      | Any page   |
| End           | Jump to last transaction       | Any page   |
| q / Esc       | Quit application               | Any page   |

---

## 🎯 Roadmap (Badge System)

### Tier 1: Foundation (5/5 complete) ✅
- ✅ Badge 1: Data Import
- ✅ Badge 2: UI Rendering
- ✅ Badge 3: Navigation (multi-page)
- ✅ Badge 4: Detail View
- ✅ Badge 5: Filters

### Tier 2: Production Pipeline (1/10)
- ✅ Badge 6: Parser Framework
- ⏭️ Badge 7: BofA Parser (NEXT)
- Badge 8-15: AppleCard, Stripe, Wise, Scotia parsers + error handling

### Tier 3: Trust Construction (0/5)
- Badge 16-20: Confidence scoring, validation, audit trails, etc.

**Total:** 6/20 badges (30%)

**Tier 1 COMPLETE!** 🎉 **Tier 2 begun!** 🏗️

---

## 🔍 Trust Features

### ✅ Active Now
- **Provenance Tracking:** source_file + line_number for every transaction
- **Idempotency:** SHA-256 hash prevents duplicates
- **Immutability:** SQLite WAL mode for crash recovery
- **Structured Logs:** created_at timestamp

### 🔄 Coming Soon
- **Confidence Scoring:** 0.0-1.0 confidence per field
- **Great Expectations:** Data quality validation
- **Provenance Viewer:** See complete audit trail
- **Explicit Verification:** User approval workflows

---

## 📈 Performance

- **Load time:** <1 second for 4,512 transactions
- **Rendering:** 60 FPS smooth scrolling
- **Memory:** ~15 MB total
- **Database:** SQLite with indexes for fast queries

---

## 🛠️ Development

### Build

```bash
cargo build --release
```

### Run tests

```bash
cargo test
```

### Project structure

```
trust-construction/
├── Cargo.toml           # Dependencies
├── README.md            # This file
├── run.sh               # Launch script
├── transactions.db      # SQLite database (gitignored)
└── src/
    ├── main.rs          # Entry point
    ├── db.rs            # Database layer
    └── ui.rs            # Terminal UI
```

---

## 📚 Documentation

- [BADGE_1_COMPLETE.md](../BADGE_1_COMPLETE.md) - Data import details
- [BADGE_2_COMPLETE.md](../BADGE_2_COMPLETE.md) - UI implementation
- [claude.md](../claude.md) - Master reference document

---

## 🎨 Design Principles

### Bloomberg Terminal-Inspired UX

1. **Consistency:** Same color = same meaning always
2. **Status Transparency:** Status bar always visible
3. **Confidence Transparency:** Show confidence scores (coming)
4. **Provenance Visibility:** Complete audit trail (coming)
5. **Explicit Verification:** User approval required (coming)

### Data-Oriented Programming

```
System = Data + Transformations + Queues + Configuration

NADA MÁS.
```

---

## 🔒 Security & Trust

- **No hardcoded credentials** - All data from CSV files
- **Idempotent operations** - Safe to run multiple times
- **ACID compliance** - WAL mode prevents corruption
- **Complete audit trail** - Every operation logged
- **Deterministic execution** - Same input = same output

---

## 📝 License

Private project - Not for redistribution

---

## 🤝 Contributing

This is a personal project, but feedback welcome!

---

**Built with Rust 🦀 + Ratatui 🐀 + SQLite 🗄️**

*Trust = Garantías, NO Esperanzas*

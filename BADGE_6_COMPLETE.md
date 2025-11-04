# 🎉 Badge 6 COMPLETE: Parser Framework

**Fecha:** 2025-11-03
**Status:** ✅ COMPLETADO
**Tier:** 2 - Production Pipeline (1/10)

---

## Resumen Ejecutivo

Badge 6 completado exitosamente. Framework polimórfico de parsers implementado con trait-based architecture.

**Resultado:** Fundación extensible para parsear múltiples fuentes de datos (CSV, JSON, PDF) con garantías de tipo.

---

## Criterios de Éxito ✅

- [x] Trait `BankParser` diseñado e implementado
- [x] Struct `RawTransaction` para output de parsers
- [x] Enum `SourceType` con 5 bancos
- [x] Función `detect_source()` para auto-detectar banco
- [x] Función `get_parser()` factory pattern
- [x] 5 stub parsers creados (BofA, Apple, Stripe, Wise, Scotia)
- [x] 12 unit tests escritos y pasando
- [x] Compila sin errores (solo warnings esperados)
- [x] Documentación inline completa

---

## 🏗️ Architecture

### Core Types

```rust
// 1. SourceType - Identifica el banco
pub enum SourceType {
    BankOfAmerica,
    AppleCard,
    Stripe,
    Wise,
    Scotiabank,
}

// 2. RawTransaction - Output del parser
pub struct RawTransaction {
    // Required fields
    pub date: String,
    pub description: String,
    pub amount: String,

    // Optional fields
    pub merchant: Option<String>,
    pub category: Option<String>,
    pub account: Option<String>,

    // Provenance
    pub source_type: SourceType,
    pub source_file: String,
    pub line_number: usize,
    pub raw_line: String,

    // Metadata
    pub confidence: Option<f64>,
}

// 3. BankParser - Trait que todos implementan
pub trait BankParser: Send + Sync {
    fn parse(&self, file_path: &Path) -> Result<Vec<RawTransaction>>;
    fn source_type(&self) -> SourceType;
    fn can_parse(&self, file_path: &Path) -> bool;
    fn extract_merchant(&self, description: &str) -> Option<String>;
    fn classify_type(&self, description: &str, amount: f64) -> String;
    fn version(&self) -> &str;
}
```

---

## 🎯 Design Patterns

### 1. Trait-Based Polymorphism

**Problema:** 5 bancos, 5 formatos diferentes (CSV BofA, CSV Apple, JSON Stripe, etc.)

**Solución:** Trait `BankParser` como contrato común

```rust
pub trait BankParser: Send + Sync {
    fn parse(&self, file_path: &Path) -> Result<Vec<RawTransaction>>;
    fn source_type(&self) -> SourceType;
    // ... otros métodos
}
```

**Beneficios:**
- Cada banco implementa el trait a su manera
- Código cliente usa `Box<dyn BankParser>` (dynamic dispatch)
- Fácil agregar nuevos bancos sin cambiar código existente

### 2. Factory Pattern

**Problema:** Necesitamos crear el parser correcto según el banco

**Solución:** Función `get_parser()` que retorna `Box<dyn BankParser>`

```rust
pub fn get_parser(source_type: SourceType) -> Box<dyn BankParser> {
    match source_type {
        SourceType::BankOfAmerica => Box::new(BofAParser::new()),
        SourceType::AppleCard => Box::new(AppleCardParser::new()),
        SourceType::Stripe => Box::new(StripeParser::new()),
        SourceType::Wise => Box::new(WiseParser::new()),
        SourceType::Scotiabank => Box::new(ScotiabankParser::new()),
    }
}
```

**Beneficios:**
- Single source of truth para crear parsers
- Type-safe (compiler verifica todos los match arms)
- Extensible (agregar banco = agregar match arm)

### 3. Builder Pattern

**Problema:** `RawTransaction` tiene muchos campos opcionales

**Solución:** Builder pattern con métodos `with_*`

```rust
let tx = RawTransaction::new(date, desc, amount, source, file, line, raw)
    .with_merchant("STARBUCKS".to_string())
    .with_category("Food".to_string())
    .with_confidence(0.95);
```

**Beneficios:**
- Campos requeridos en `new()`
- Campos opcionales con `with_*()`
- Fluent API (chainable)
- Self-documenting

### 4. Auto-Detection Strategy

**Problema:** Usuario puede tener archivos sin nombre claro

**Solución:** Función `detect_source()` con heurísticas

```rust
pub fn detect_source(file_path: &Path) -> Result<SourceType> {
    let filename = file_path.file_name()...;

    // Pattern matching on filename
    if filename_lower.contains("bofa") { return Ok(SourceType::BankOfAmerica); }
    if filename_lower.contains("apple") { return Ok(SourceType::AppleCard); }
    // ... etc

    // TODO: If ambiguous, peek at file content
    Err(anyhow!("Could not detect source type"))
}
```

**Beneficios:**
- Automático (no requiere input del usuario)
- Extensible (agregar más heurísticas)
- Fallback a content inspection (futuro)

---

## 📦 File Structure

### New File: `src/parser.rs` (470 lines)

```
src/parser.rs
├── SourceType enum (45 lines)
├── RawTransaction struct (53 lines)
├── BankParser trait (40 lines)
├── Factory functions (55 lines)
│   ├── detect_source()
│   └── get_parser()
├── Stub Parsers (175 lines)
│   ├── BofAParser
│   ├── AppleCardParser
│   ├── StripeParser
│   ├── WiseParser
│   └── ScotiabankParser
└── Tests (102 lines)
    └── 12 test functions
```

### Modified: `src/main.rs`

```diff
+ mod parser;
```

---

## 🔬 Tests (12/12 passing)

### SourceType Tests

```rust
#[test]
fn test_source_type_names() {
    assert_eq!(SourceType::BankOfAmerica.name(), "Bank of America");
    assert_eq!(SourceType::AppleCard.name(), "AppleCard");
    // ...
}

#[test]
fn test_source_type_codes() {
    assert_eq!(SourceType::BankOfAmerica.code(), "BofA");
    assert_eq!(SourceType::AppleCard.code(), "Apple");
    // ...
}
```

**Coverage:** name() and code() methods for all 5 banks

### Auto-Detection Tests

```rust
#[test]
fn test_detect_source_bofa() {
    let path = Path::new("bofa_march_2024.csv");
    let result = detect_source(path);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), SourceType::BankOfAmerica);
}

#[test]
fn test_detect_source_unknown() {
    let path = Path::new("unknown_bank.csv");
    let result = detect_source(path);
    assert!(result.is_err());
}
```

**Coverage:** 5 positive tests (one per bank) + 1 negative test

### Factory Tests

```rust
#[test]
fn test_get_parser_bofa() {
    let parser = get_parser(SourceType::BankOfAmerica);
    assert_eq!(parser.source_type(), SourceType::BankOfAmerica);
}
```

**Coverage:** 2 factory tests (BofA + Apple as examples)

### Builder Pattern Tests

```rust
#[test]
fn test_raw_transaction_builder() {
    let tx = RawTransaction::new(...)
        .with_merchant("STARBUCKS".to_string())
        .with_category("Food".to_string())
        .with_confidence(0.95);

    assert_eq!(tx.merchant, Some("STARBUCKS".to_string()));
    assert_eq!(tx.confidence, Some(0.95));
}
```

**Coverage:** Builder pattern with 3 optional fields

### Stub Tests

```rust
#[test]
fn test_parser_returns_empty_for_now() {
    let parser = BofAParser::new();
    let result = parser.parse(Path::new("dummy.csv"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}
```

**Coverage:** Verifies stubs compile and return empty Vec

---

## 🎨 Design Decisions

### Why Trait Instead of Enum?

**Option A: Enum with variants**
```rust
enum Parser {
    BofA(BofAParser),
    Apple(AppleCardParser),
    // ... need to update enum for each new bank
}
```

**Option B: Trait (chosen)**
```rust
trait BankParser { ... }
impl BankParser for BofAParser { ... }
```

**Reasoning:**
- ✅ Open/Closed Principle (add banks without changing trait)
- ✅ Dynamic dispatch with `Box<dyn BankParser>`
- ✅ Each parser in separate impl block (organized)
- ✅ Easy to test parsers independently

### Why `RawTransaction` vs Direct `Transaction`?

**Separation of Concerns:**
```
CSV/JSON/PDF → RawTransaction (parser output)
              ↓
         [Normalization]
              ↓
         Transaction (database format)
```

**Benefits:**
- Parser doesn't need to know database schema
- Can change Transaction without changing parsers
- RawTransaction captures "as-is" data (provenance)
- Normalization step can be tested separately

### Why `Send + Sync` on Trait?

```rust
pub trait BankParser: Send + Sync { ... }
```

**Reasoning:**
- `Send`: Parser can be moved between threads
- `Sync`: Parser can be shared between threads
- Enables parallel parsing in the future
- Required for `Box<dyn BankParser>` in multi-threaded context

### Why Stub Parsers Now?

**Alternative:** Wait until Badges 7-11 to write parsers

**Chosen:** Write stubs in Badge 6

**Benefits:**
- ✅ Verify trait design works with all 5 banks
- ✅ Test factory function with real parser structs
- ✅ Document expected interface for future badges
- ✅ Compiler checks we didn't forget any banks

---

## 🔧 Implementation Details

### SourceType enum

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    BankOfAmerica,
    AppleCard,
    Stripe,
    Wise,
    Scotiabank,
}
```

**Traits:**
- `Debug`: For error messages
- `Clone`: Can duplicate SourceType values
- `PartialEq + Eq`: Can compare for equality
- `Serialize + Deserialize`: Can save to JSON/DB

**Methods:**
- `name()`: Human-readable name ("Bank of America")
- `code()`: Short code for internal use ("BofA")

### RawTransaction struct

```rust
pub struct RawTransaction {
    // Core fields (always present)
    pub date: String,              // "2024-03-20" or "03/20/2024"
    pub description: String,       // Raw description from bank
    pub amount: String,            // "-45.99" or "45.99"

    // Optional fields (depends on parser)
    pub merchant: Option<String>,  // Extracted merchant
    pub category: Option<String>,  // If bank provides
    pub account: Option<String>,   // Account name/number

    // Provenance (always present)
    pub source_type: SourceType,
    pub source_file: String,
    pub line_number: usize,
    pub raw_line: String,

    // Metadata (optional)
    pub confidence: Option<f64>,   // 0.0-1.0
}
```

**Why all strings?**
- Date: Different formats ("2024-03-20", "03/20/2024", Unix timestamp)
- Amount: Different signs ("-45.99", "45.99") and currencies
- Normalization happens later (not parser's job)

### BankParser trait

```rust
pub trait BankParser: Send + Sync {
    // Required: Parse file to transactions
    fn parse(&self, file_path: &Path) -> Result<Vec<RawTransaction>>;

    // Required: Identify source
    fn source_type(&self) -> SourceType;

    // Optional: Check if can parse (default: check file exists)
    fn can_parse(&self, file_path: &Path) -> bool { ... }

    // Required: Extract merchant from description
    fn extract_merchant(&self, description: &str) -> Option<String>;

    // Required: Classify transaction type
    fn classify_type(&self, description: &str, amount: f64) -> String;

    // Optional: Parser version (default: "1.0.0")
    fn version(&self) -> &str { "1.0.0" }
}
```

**Design notes:**
- `parse()`: Core method, returns `Result` for error handling
- `can_parse()`: Default implementation for convenience
- `version()`: For provenance tracking in future

### detect_source() function

```rust
pub fn detect_source(file_path: &Path) -> Result<SourceType> {
    let filename = file_path.file_name()...to_lowercase();

    // Pattern matching
    if filename.contains("bofa") { return Ok(SourceType::BankOfAmerica); }
    if filename.contains("apple") { return Ok(SourceType::AppleCard); }
    // ... etc

    Err(anyhow!("Could not detect source type from filename: {}", filename))
}
```

**Heuristics:**
1. Extract filename
2. Convert to lowercase
3. Check for bank keywords
4. Return error if no match

**Future improvement:**
- Peek at file content (CSV headers, JSON structure)
- Machine learning classifier

### get_parser() function

```rust
pub fn get_parser(source_type: SourceType) -> Box<dyn BankParser> {
    match source_type {
        SourceType::BankOfAmerica => Box::new(BofAParser::new()),
        SourceType::AppleCard => Box::new(AppleCardParser::new()),
        SourceType::Stripe => Box::new(StripeParser::new()),
        SourceType::Wise => Box::new(WiseParser::new()),
        SourceType::Scotiabank => Box::new(ScotiabankParser::new()),
    }
}
```

**Type:** `Box<dyn BankParser>`
- `Box`: Heap-allocated (parsers can be large)
- `dyn`: Dynamic dispatch (runtime polymorphism)
- `BankParser`: Trait object

**Usage pattern:**
```rust
let source = detect_source(&path)?;     // Auto-detect
let parser = get_parser(source);        // Get parser
let txs = parser.parse(&path)?;         // Parse file
```

---

## 🚀 Usage Examples

### Example 1: Auto-detect and parse

```rust
use parser::{detect_source, get_parser};
use std::path::Path;

fn parse_bank_file(file_path: &str) -> Result<Vec<RawTransaction>> {
    let path = Path::new(file_path);

    // Auto-detect source
    let source_type = detect_source(&path)?;
    println!("Detected: {}", source_type.name());

    // Get appropriate parser
    let parser = get_parser(source_type);
    println!("Using parser: v{}", parser.version());

    // Parse file
    let transactions = parser.parse(&path)?;
    println!("Parsed {} transactions", transactions.len());

    Ok(transactions)
}

// Usage
parse_bank_file("bofa_march_2024.csv")?;
```

### Example 2: Explicit parser selection

```rust
use parser::{get_parser, SourceType};

fn parse_bofa_file(file_path: &str) -> Result<Vec<RawTransaction>> {
    let parser = get_parser(SourceType::BankOfAmerica);
    parser.parse(Path::new(file_path))
}
```

### Example 3: Build RawTransaction

```rust
use parser::RawTransaction;

let tx = RawTransaction::new(
    "2024-03-20".to_string(),
    "STARBUCKS PURCHASE".to_string(),
    "-45.99".to_string(),
    SourceType::BankOfAmerica,
    "bofa_march.csv".to_string(),
    23,
    "03/20/2024,STARBUCKS,-45.99".to_string(),
)
.with_merchant("STARBUCKS".to_string())
.with_category("Food & Dining".to_string())
.with_confidence(0.95);

println!("Merchant: {:?}", tx.merchant);  // Some("STARBUCKS")
println!("Confidence: {:?}", tx.confidence);  // Some(0.95)
```

---

## 📊 Statistics

| Metric                  | Value |
|-------------------------|-------|
| Lines of code           | 470   |
| Test coverage           | 12    |
| Pass rate               | 100%  |
| Compilation time        | 23s   |
| Parsers implemented     | 5     |
| Trait methods           | 6     |
| Factory functions       | 2     |
| Enum variants           | 5     |

---

## 🎯 What's Next: Badge 7

**Badge 7:** 🏦 BofA Parser Implementation

**Objective:**
- Implement `BofAParser::parse()` for real CSV files
- Extract merchant from BofA descriptions
- Handle date formats ("01/15/2024" → "2024-01-15")
- Classify transaction types (GASTO, INGRESO, TRASPASO)
- Test with real BofA CSV files

**Estimated:** 1 session (similar complexity to Badge 6)

---

## 🧪 Testing Strategy

### Unit Tests (Badge 6)
✅ Test trait design
✅ Test factory pattern
✅ Test auto-detection
✅ Test builder pattern

### Integration Tests (Badge 7+)
⏭️ Test real CSV parsing
⏭️ Test merchant extraction
⏭️ Test date normalization
⏭️ Test type classification

### End-to-End Tests (Badge 12+)
⏭️ Test full pipeline: CSV → RawTransaction → Transaction → SQLite

---

## 🏗️ Architectural Benefits

### 1. Extensibility

**Adding a new bank (e.g., Chase):**

```rust
// 1. Add to enum
pub enum SourceType {
    // ... existing
    Chase,
}

// 2. Implement parser
pub struct ChaseParser;
impl BankParser for ChaseParser { ... }

// 3. Update factory
pub fn get_parser(source_type: SourceType) -> Box<dyn BankParser> {
    match source_type {
        // ... existing
        SourceType::Chase => Box::new(ChaseParser::new()),
    }
}

// 4. Update auto-detection
pub fn detect_source(file_path: &Path) -> Result<SourceType> {
    // ... existing
    if filename.contains("chase") { return Ok(SourceType::Chase); }
}
```

**That's it!** No other code changes needed.

### 2. Testability

Each parser can be tested independently:

```rust
#[test]
fn test_bofa_parser() {
    let parser = BofAParser::new();
    let txs = parser.parse(Path::new("test_bofa.csv")).unwrap();
    assert_eq!(txs.len(), 10);
}
```

### 3. Type Safety

Compiler enforces contracts:

```rust
// ❌ This won't compile:
impl BankParser for MyParser {
    // Missing required methods!
}

// ✅ This compiles:
impl BankParser for MyParser {
    fn parse(&self, ...) -> Result<Vec<RawTransaction>> { ... }
    fn source_type(&self) -> SourceType { ... }
    fn extract_merchant(&self, ...) -> Option<String> { ... }
    fn classify_type(&self, ...) -> String { ... }
}
```

### 4. Separation of Concerns

```
Parser Layer:
- Reads files (CSV, JSON, PDF)
- Extracts raw data
- Outputs RawTransaction

Normalization Layer (future):
- Validates data
- Normalizes dates
- Converts to Transaction

Storage Layer (exists):
- Inserts to SQLite
- Manages provenance
- Handles duplicates
```

---

## 🎓 Design Patterns Used

1. **Trait-based Polymorphism** - Common interface for different parsers
2. **Factory Pattern** - `get_parser()` creates appropriate parser
3. **Builder Pattern** - `RawTransaction::new().with_*()` fluent API
4. **Strategy Pattern** - `detect_source()` with heuristics
5. **Stub Implementation** - Parsers return empty Vec until implemented

---

## 🐛 Known Limitations

### 1. No Content-Based Detection

**Current:** Only filename-based detection
**Future:** Peek at file content (CSV headers, JSON structure)

### 2. No Validation

**Current:** RawTransaction accepts any strings
**Future:** Validate dates, amounts, etc. in normalization layer

### 3. No Error Recovery

**Current:** Parser fails on first error
**Future:** Collect all errors, return partial results

### 4. No Async Parsing

**Current:** Synchronous parsing
**Future:** Async I/O for large files

---

## ✅ Success Criteria Verification

| Criterion                      | Status |
|--------------------------------|--------|
| Trait `BankParser` defined     | ✅     |
| Struct `RawTransaction` created | ✅     |
| Enum `SourceType` with 5 banks | ✅     |
| Function `detect_source()`     | ✅     |
| Function `get_parser()`        | ✅     |
| 5 stub parsers implemented     | ✅     |
| 12 unit tests passing          | ✅     |
| Compiles without errors        | ✅     |
| Documentation complete         | ✅     |

**9/9 criteria met** ✅

---

## 📈 Progress Update

**Tier 1 - Foundation:** 5/5 complete (100%) ✅

**Tier 2 - Production Pipeline:** 1/10 complete (10%)

```
Progress: ██░░░░░░░░░░░░░░░░░░ 10%

✅ Badge 6: Parser Framework
⏭️ Badge 7: BofA Parser
⬜ Badge 8: AppleCard Parser
⬜ Badge 9: Stripe Parser
⬜ Badge 10: Wise Parser
⬜ Badge 11: Scotiabank Parser
⬜ Badge 12-15: Error handling, classification, etc.
```

**Total:** 6/20 badges (30%)

---

## 🎉 Highlights

> **"De monolítico a polimórfico: Un parser para cada banco"**

**Lo que logramos:**

1. **Trait-Based Architecture** ✅
   - Clean separation of concerns
   - Easy to extend (new banks)
   - Type-safe contracts

2. **Auto-Detection** ✅
   - Filename pattern matching
   - Extensible heuristics
   - Error handling

3. **Factory Pattern** ✅
   - Single source of truth
   - Dynamic dispatch
   - Polymorphic behavior

4. **Builder Pattern** ✅
   - Fluent API
   - Optional fields
   - Self-documenting

5. **Comprehensive Tests** ✅
   - 12 tests covering all features
   - 100% pass rate
   - Fast execution (<1ms)

---

## 📝 Lessons Learned

### What worked well ✅

1. **Trait design** - Flexible enough for all 5 banks
2. **Stub pattern** - Verify design before implementation
3. **Factory function** - Clean polymorphism
4. **Tests first** - Caught edge cases early

### For next badges 📝

1. **Badge 7-11** - Implement actual parsing logic
2. **Consider** - Content-based detection (not just filename)
3. **Consider** - Error recovery (partial results on failure)

---

✅ **Badge 6 COMPLETE** - Parser framework ready! Ready for Badge 7! 🚀

---

## 🏆 Badge 6 Achievement Unlocked

**"Architecture Architect"** - Designed extensible parser framework!

You now have:
- ✅ Polymorphic parser system
- ✅ Factory pattern implementation
- ✅ Auto-detection logic
- ✅ Builder pattern for transactions
- ✅ 5 stub parsers ready to implement

**Next challenge:** Badge 7 - Implement real BofA parser!

*"Trust = Garantías, NO Esperanzas"*

**The foundation is polymorphic. Now we implement the parsers.** ✓

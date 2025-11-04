// 🏗️ Parser Framework - Badge 6
// Polymorphic parser system for 5 banks

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ============================================================================
// CORE TYPES
// ============================================================================

/// SourceType - Identifica de qué banco viene el documento
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    BankOfAmerica,
    AppleCard,
    Stripe,
    Wise,
    Scotiabank,
}

impl SourceType {
    /// Human-readable name for display
    pub fn name(&self) -> &str {
        match self {
            SourceType::BankOfAmerica => "Bank of America",
            SourceType::AppleCard => "AppleCard",
            SourceType::Stripe => "Stripe",
            SourceType::Wise => "Wise",
            SourceType::Scotiabank => "Scotiabank",
        }
    }

    /// Short code for internal use
    pub fn code(&self) -> &str {
        match self {
            SourceType::BankOfAmerica => "BofA",
            SourceType::AppleCard => "Apple",
            SourceType::Stripe => "Stripe",
            SourceType::Wise => "Wise",
            SourceType::Scotiabank => "Scotia",
        }
    }
}

/// RawTransaction - Output of parser.parse()
/// Esta es la representación "cruda" antes de normalizar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTransaction {
    // Core fields (todos los parsers deben proveer)
    pub date: String,              // Date in any format (parser-specific)
    pub description: String,       // Raw description from source
    pub amount: String,            // Amount as string (could be "45.99" or "-45.99")

    // Optional fields (depende del parser)
    pub merchant: Option<String>,  // Extracted merchant name
    pub category: Option<String>,  // If source provides category
    pub account: Option<String>,   // Account name/number

    // Provenance (siempre presente)
    pub source_type: SourceType,   // Which bank
    pub source_file: String,       // Original filename
    pub line_number: usize,        // Line in original file

    // Metadata (parser puede añadir)
    pub raw_line: String,          // Original line for debugging
    pub confidence: Option<f64>,   // Parser confidence (0.0-1.0)
}

impl RawTransaction {
    /// Create a new RawTransaction with required fields
    pub fn new(
        date: String,
        description: String,
        amount: String,
        source_type: SourceType,
        source_file: String,
        line_number: usize,
        raw_line: String,
    ) -> Self {
        RawTransaction {
            date,
            description,
            amount,
            merchant: None,
            category: None,
            account: None,
            source_type,
            source_file,
            line_number,
            raw_line,
            confidence: None,
        }
    }

    /// Builder pattern: add optional merchant
    pub fn with_merchant(mut self, merchant: String) -> Self {
        self.merchant = Some(merchant);
        self
    }

    /// Builder pattern: add optional category
    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    /// Builder pattern: add optional account
    pub fn with_account(mut self, account: String) -> Self {
        self.account = Some(account);
        self
    }

    /// Builder pattern: add confidence score
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }
}

// ============================================================================
// COMPOSABLE TRAITS - Expression Problem Solution
// ============================================================================

/// BankParser - Core trait (minimal, required)
///
/// Esta es la única interfaz OBLIGATORIA. Todo lo demás es opcional.
///
/// Expression Problem Coverage:
/// - Agregar TIPOS (bancos): Implementar este trait → No toca código existente ✓
/// - Agregar FUNCIONES: Crear nuevo trait → No toca parsers existentes ✓
pub trait BankParser: Send + Sync {
    /// Parse a file and return raw transactions
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to parse (CSV, JSON, etc.)
    ///
    /// # Returns
    /// * `Ok(Vec<RawTransaction>)` - List of raw transactions
    /// * `Err(anyhow::Error)` - If parsing fails
    fn parse(&self, file_path: &Path) -> Result<Vec<RawTransaction>>;

    /// Get the source type this parser handles
    fn source_type(&self) -> SourceType;

    /// Get parser version (for provenance tracking)
    fn version(&self) -> &str {
        "1.0.0"
    }
}

/// FileValidator - Optional capability: Check if parser can handle file
///
/// Extensión OPCIONAL. Parsers que no lo implementan = asumen que pueden parsear.
pub trait FileValidator {
    /// Check if this parser can handle a given file
    ///
    /// Can check:
    /// - File extension (.csv, .json, .pdf)
    /// - File content (CSV headers, JSON structure)
    /// - File size (skip empty files)
    fn can_parse(&self, file_path: &Path) -> bool;
}

/// MerchantExtractor - Optional capability: Extract merchant from description
///
/// Extensión OPCIONAL. Parsers que no lo implementan = merchant queda en description.
pub trait MerchantExtractor {
    /// Extract merchant name from description
    ///
    /// Each bank has different formats:
    /// - BofA: "DEBIT PURCHASE -VISA STARBUCKS" → "STARBUCKS"
    /// - AppleCard: Already clean
    /// - Stripe: metadata.customer_name
    fn extract_merchant(&self, description: &str) -> Option<String>;
}

/// TypeClassifier - Optional capability: Classify transaction type
///
/// Extensión OPCIONAL. Parsers que no lo implementan = tipo queda como "UNKNOWN".
pub trait TypeClassifier {
    /// Classify transaction type
    ///
    /// Returns: "GASTO", "INGRESO", "PAGO_TARJETA", "TRASPASO"
    fn classify_type(&self, description: &str, amount: f64) -> String;
}

// ============================================================================
// FUTURE EXTENSIONS (examples - not implemented yet)
// ============================================================================

/// AmountValidator - Future extension: Validate amounts
///
/// Ejemplo de cómo agregar NUEVAS FUNCIONES sin tocar código existente.
/// Los parsers existentes NO necesitan implementar esto.
pub trait AmountValidator {
    fn validate_amount(&self, amount: &str) -> Result<f64>;
}

/// DateNormalizer - Future extension: Normalize dates
///
/// Otro ejemplo de extensión futura.
pub trait DateNormalizer {
    fn normalize_date(&self, date: &str) -> Result<String>;
}

/// CategoryInferrer - Future extension: Infer categories from ML
///
/// Otro ejemplo más.
pub trait CategoryInferrer {
    fn infer_category(&self, merchant: &str, amount: f64) -> Option<String>;
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/// Detect source type from filename or file content
///
/// # Strategy:
/// 1. Check filename patterns (e.g., "bofa_*.csv" → BankOfAmerica)
/// 2. If ambiguous, peek at file content (e.g., CSV headers)
/// 3. Return error if can't determine
///
/// # Examples:
/// ```
/// detect_source("bofa_march_2024.csv") → SourceType::BankOfAmerica
/// detect_source("Apple Card Activity.csv") → SourceType::AppleCard
/// detect_source("stripe_january.json") → SourceType::Stripe
/// ```
pub fn detect_source(file_path: &Path) -> Result<SourceType> {
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let filename_lower = filename.to_lowercase();

    // Pattern matching on filename
    if filename_lower.contains("bofa") || filename_lower.contains("bank_of_america") {
        return Ok(SourceType::BankOfAmerica);
    }

    if filename_lower.contains("apple") {
        return Ok(SourceType::AppleCard);
    }

    if filename_lower.contains("stripe") {
        return Ok(SourceType::Stripe);
    }

    if filename_lower.contains("wise") {
        return Ok(SourceType::Wise);
    }

    if filename_lower.contains("scotia") {
        return Ok(SourceType::Scotiabank);
    }

    // TODO: If filename is ambiguous, peek at file content
    // For now, return error
    Err(anyhow::anyhow!(
        "Could not detect source type from filename: {}",
        filename
    ))
}

/// Get appropriate parser for a source type
///
/// Factory pattern: Returns Box<dyn BankParser> for polymorphism
///
/// # Example:
/// ```
/// let source = detect_source("bofa_march.csv")?;
/// let parser = get_parser(source);
/// let transactions = parser.parse("bofa_march.csv")?;
/// ```
pub fn get_parser(source_type: SourceType) -> Box<dyn BankParser> {
    match source_type {
        SourceType::BankOfAmerica => Box::new(BofAParser::new()),
        SourceType::AppleCard => Box::new(AppleCardParser::new()),
        SourceType::Stripe => Box::new(StripeParser::new()),
        SourceType::Wise => Box::new(WiseParser::new()),
        SourceType::Scotiabank => Box::new(ScotiabankParser::new()),
    }
}

// ============================================================================
// STUB PARSERS (will be implemented in future badges)
// ============================================================================

/// Bank of America Parser (Badge 7)
pub struct BofAParser;

impl BofAParser {
    pub fn new() -> Self {
        BofAParser
    }
}

// Core trait (required)
impl BankParser for BofAParser {
    fn parse(&self, file_path: &Path) -> Result<Vec<RawTransaction>> {
        use csv::ReaderBuilder;
        use std::fs::File;

        let file = File::open(file_path)
            .with_context(|| format!("Failed to open file: {}", file_path.display()))?;

        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        let mut transactions = Vec::new();
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.csv")
            .to_string();

        for (line_num, result) in reader.records().enumerate() {
            let record = result.with_context(|| {
                format!("Failed to parse CSV line {} in {}", line_num + 2, filename)
            })?;

            // BofA CSV format: Date,Description,Amount
            // Example: "12/31/2024","Stripe, Des:transfer, Id:st-...","-$855.94"
            let date = record.get(0).unwrap_or("").to_string();
            let description = record.get(1).unwrap_or("").to_string();
            let amount = record.get(2).unwrap_or("").to_string();

            let raw_line = format!("{},{},{}", date, description, amount);

            let tx = RawTransaction::new(
                date,
                description.clone(),
                amount,
                SourceType::BankOfAmerica,
                filename.clone(),
                line_num + 2, // +2 because: 1-indexed + header row
                raw_line,
            );

            // Extract merchant if possible
            let merchant = self.extract_merchant(&description);
            let tx = if let Some(m) = merchant {
                tx.with_merchant(m)
            } else {
                tx
            };

            transactions.push(tx);
        }

        Ok(transactions)
    }

    fn source_type(&self) -> SourceType {
        SourceType::BankOfAmerica
    }
}

// Optional: MerchantExtractor
impl MerchantExtractor for BofAParser {
    fn extract_merchant(&self, description: &str) -> Option<String> {
        // BofA patterns:
        // "Stripe, Des:transfer, Id:st-..." → "Stripe"
        // "Wise Us Inc, Des:thera Pay, ..." → "Wise"
        // "Bank of America Credit Card Bill Payment" → "Bank of America"
        // "Applecard Gsbank Des:payment, ..." → "Applecard Gsbank"

        let desc = description.trim();

        // Pattern 1: "Merchant, Des:..."
        if let Some(comma_pos) = desc.find(',') {
            let merchant = desc[..comma_pos].trim();
            if !merchant.is_empty() {
                return Some(merchant.to_string());
            }
        }

        // Pattern 2: Take first significant word
        let first_word = desc.split_whitespace().next()?;
        if first_word.len() > 2 {
            Some(first_word.to_string())
        } else {
            None
        }
    }
}

// Optional: TypeClassifier
impl TypeClassifier for BofAParser {
    fn classify_type(&self, description: &str, amount: f64) -> String {
        let desc_lower = description.to_lowercase();

        // Credit card payment
        if desc_lower.contains("credit card") || desc_lower.contains("bill payment") {
            return "PAGO_TARJETA".to_string();
        }

        // Transfers (check FIRST - most specific)
        if desc_lower.contains("des:transfer") {
            return "TRASPASO".to_string();
        }

        // Income (positive amounts or deposits)
        if amount > 0.0 || desc_lower.contains("deposit") || desc_lower.contains("des:thera pay") {
            return "INGRESO".to_string();
        }

        // Default: expense
        "GASTO".to_string()
    }
}

/// AppleCard Parser (Badge 8)
pub struct AppleCardParser;

impl AppleCardParser {
    pub fn new() -> Self {
        AppleCardParser
    }
}

impl BankParser for AppleCardParser {
    fn parse(&self, file_path: &Path) -> Result<Vec<RawTransaction>> {
        use csv::ReaderBuilder;
        use std::fs::File;

        let file = File::open(file_path)
            .with_context(|| format!("Failed to open file: {}", file_path.display()))?;

        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        let mut transactions = Vec::new();
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.csv")
            .to_string();

        for (line_num, result) in reader.records().enumerate() {
            let record = result.with_context(|| {
                format!("Failed to parse CSV line {} in {}", line_num + 2, filename)
            })?;

            // AppleCard CSV format: Date,Description,Amount,Category,Merchant
            // Example: "10/26/2024","UBER *EATS MR TREUBLAAN...","3.74","Restaurants","Uber Eats"
            let date = record.get(0).unwrap_or("").to_string();
            let description = record.get(1).unwrap_or("").to_string();
            let amount = record.get(2).unwrap_or("").to_string();
            let category = record.get(3).map(|s| s.to_string());
            let merchant = record.get(4).map(|s| s.to_string());

            let raw_line = format!("{},{},{}", date, description, amount);

            let mut tx = RawTransaction::new(
                date,
                description.clone(),
                amount,
                SourceType::AppleCard,
                filename.clone(),
                line_num + 2,
                raw_line,
            );

            // AppleCard provides clean merchant name
            if let Some(m) = merchant {
                tx = tx.with_merchant(m);
            }

            // Category if available
            if let Some(c) = category {
                tx = tx.with_category(c);
            }

            transactions.push(tx);
        }

        Ok(transactions)
    }

    fn source_type(&self) -> SourceType {
        SourceType::AppleCard
    }
}

impl MerchantExtractor for AppleCardParser {
    fn extract_merchant(&self, description: &str) -> Option<String> {
        // AppleCard: Merchant already clean in separate column
        // But from description, extract first words before location

        let desc = description.trim();

        // Pattern: "UBER *EATS MR TREUBLAAN 7 AMSTERDAM..." → "UBER *EATS"
        // Look for location indicators: numbers followed by city names
        let words: Vec<&str> = desc.split_whitespace().collect();

        if words.is_empty() {
            return None;
        }

        // Take first 2-3 words as merchant
        let merchant = if words.len() >= 2 {
            format!("{} {}", words[0], words[1])
        } else {
            words[0].to_string()
        };

        Some(merchant)
    }
}

impl TypeClassifier for AppleCardParser {
    fn classify_type(&self, description: &str, _amount: f64) -> String {
        let desc_lower = description.to_lowercase();

        // Payments (ACH deposits are payments from bank)
        if desc_lower.contains("ach deposit") || desc_lower.contains("payment") {
            return "PAGO_TARJETA".to_string();
        }

        // All other transactions on credit card are expenses
        // (AppleCard is a credit card, so charges are expenses)
        "GASTO".to_string()
    }
}

/// Stripe Parser (Badge 9)
pub struct StripeParser;

impl StripeParser {
    pub fn new() -> Self {
        StripeParser
    }
}

impl BankParser for StripeParser {
    fn parse(&self, file_path: &Path) -> Result<Vec<RawTransaction>> {
        use serde_json::Value;
        use std::fs::File;
        use std::io::BufReader;

        let file = File::open(file_path)
            .with_context(|| format!("Failed to open file: {}", file_path.display()))?;

        let reader = BufReader::new(file);
        let json: Value = serde_json::from_reader(reader)
            .with_context(|| format!("Failed to parse JSON from {}", file_path.display()))?;

        let mut transactions = Vec::new();
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.json")
            .to_string();

        // Stripe API returns { "data": [...], "object": "list" }
        let data = json
            .get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| anyhow::anyhow!("JSON missing 'data' array"))?;

        for (idx, item) in data.iter().enumerate() {
            // Stripe balance_transaction format:
            // {
            //   "id": "txn_...",
            //   "amount": 286770,  // in cents
            //   "created": 1735084800,  // Unix timestamp
            //   "currency": "usd",
            //   "description": "Payment from eugenio Castro Garza",
            //   "type": "payout"
            // }

            let id = item.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let amount_cents = item.get("amount")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            // Convert cents to dollars
            let amount_dollars = amount_cents as f64 / 100.0;
            let amount_str = format!("{:.2}", amount_dollars);

            let created_timestamp = item.get("created")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            // Convert Unix timestamp to date string
            use chrono::{DateTime, Utc};
            let datetime = DateTime::<Utc>::from_timestamp(created_timestamp, 0)
                .ok_or_else(|| anyhow::anyhow!("Invalid timestamp: {}", created_timestamp))?;
            let date = datetime.format("%m/%d/%Y").to_string();

            let description = item.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let tx_type = item.get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let raw_line = serde_json::to_string(item)
                .unwrap_or_else(|_| "{}".to_string());

            let full_description = if description.is_empty() {
                format!("Stripe {} (ID: {})", tx_type, id)
            } else {
                format!("{} (ID: {})", description, id)
            };

            let tx = RawTransaction::new(
                date,
                full_description.clone(),
                amount_str,
                SourceType::Stripe,
                filename.clone(),
                idx + 1, // JSON array index (1-based for consistency)
                raw_line,
            );

            // Extract merchant from description
            let merchant = self.extract_merchant(&description);
            let tx = if let Some(m) = merchant {
                tx.with_merchant(m)
            } else {
                tx
            };

            transactions.push(tx);
        }

        Ok(transactions)
    }

    fn source_type(&self) -> SourceType {
        SourceType::Stripe
    }
}

impl MerchantExtractor for StripeParser {
    fn extract_merchant(&self, description: &str) -> Option<String> {
        // Stripe description patterns:
        // "Payment from eugenio Castro Garza" → "eugenio Castro Garza"
        // "Subscription creation" → "Subscription"
        // "Charge for invoice" → None (generic)

        if description.is_empty() {
            return None;
        }

        // Pattern 1: "Payment from X" → X
        if let Some(from_pos) = description.find("from ") {
            let merchant = description[from_pos + 5..].trim();
            if !merchant.is_empty() {
                return Some(merchant.to_string());
            }
        }

        // Pattern 2: "Payment to X" → X
        if let Some(to_pos) = description.find("to ") {
            let merchant = description[to_pos + 3..].trim();
            if !merchant.is_empty() {
                return Some(merchant.to_string());
            }
        }

        // Pattern 3: Take first word if significant
        let first_word = description.split_whitespace().next()?;
        if first_word.len() > 3 {
            Some(first_word.to_string())
        } else {
            None
        }
    }
}

impl TypeClassifier for StripeParser {
    fn classify_type(&self, description: &str, _amount: f64) -> String {
        let desc_lower = description.to_lowercase();

        // Stripe is payment processor - mostly income (payouts)
        // But could have refunds, fees, etc.

        // Refunds = expenses
        if desc_lower.contains("refund") {
            return "GASTO".to_string();
        }

        // Fees = expenses
        if desc_lower.contains("fee") || desc_lower.contains("charge") {
            return "GASTO".to_string();
        }

        // Default: payouts are income
        "INGRESO".to_string()
    }
}

/// Wise Parser (Badge 10)
pub struct WiseParser;

impl WiseParser {
    pub fn new() -> Self {
        WiseParser
    }
}

impl BankParser for WiseParser {
    fn parse(&self, file_path: &Path) -> Result<Vec<RawTransaction>> {
        use csv::ReaderBuilder;
        use std::fs::File;

        let file = File::open(file_path)
            .with_context(|| format!("Failed to open file: {}", file_path.display()))?;

        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        let mut transactions = Vec::new();
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.csv")
            .to_string();

        for (line_num, result) in reader.records().enumerate() {
            let record = result.with_context(|| {
                format!("Failed to parse CSV line {} in {}", line_num + 2, filename)
            })?;

            // Wise CSV format: TransferWise ID, Date, Amount, Currency, Description, Payee Name, Exchange Rate, Fee Amount, Total Amount
            // Example: "TRANSFER-123456","12/31/2024","2000.00","USD","Payment from Bloom","Bloom Financial",1.00,0.00,2000.00

            let id = record.get(0).unwrap_or("").to_string();
            let date = record.get(1).unwrap_or("").to_string();
            let amount_str = record.get(2).unwrap_or("").to_string();
            let currency = record.get(3).unwrap_or("USD").to_string();
            let description = record.get(4).unwrap_or("").to_string();
            let payee_name = record.get(5).unwrap_or("").to_string();
            let exchange_rate_str = record.get(6).unwrap_or("1.0");
            let fee_str = record.get(7).unwrap_or("0.0");

            // Parse amount
            let amount = amount_str.trim().parse::<f64>()
                .unwrap_or_else(|_| {
                    // Try removing commas
                    amount_str.replace(",", "").parse::<f64>().unwrap_or(0.0)
                });

            // Parse exchange rate
            let exchange_rate = exchange_rate_str.trim().parse::<f64>().unwrap_or(1.0);

            // Parse fee (for future use)
            let _fee = fee_str.trim().parse::<f64>().unwrap_or(0.0);

            // Convert to USD if needed
            let amount_usd = if currency == "USD" {
                amount
            } else if currency == "EUR" {
                // EUR to USD: divide by exchange rate (EUR/USD rate)
                amount / exchange_rate
            } else if currency == "MXN" {
                // MXN to USD: divide by exchange rate (MXN/USD rate)
                amount / exchange_rate
            } else {
                // Unknown currency, use exchange rate as is
                amount / exchange_rate
            };

            let amount_usd_str = format!("{:.2}", amount_usd.abs());

            let raw_line = format!("{},{},{},{},{}", id, date, amount_str, currency, description);

            // Build full description with currency info
            let full_description = if currency != "USD" {
                format!("{} ({} {} → ${:.2} USD @ rate {:.4})",
                    description, amount.abs(), currency, amount_usd.abs(), exchange_rate)
            } else {
                format!("{} (ID: {})", description, id)
            };

            let tx = RawTransaction::new(
                date,
                full_description.clone(),
                amount_usd_str,
                SourceType::Wise,
                filename.clone(),
                line_num + 2,
                raw_line,
            );

            // Extract merchant from payee_name or description
            let merchant = if !payee_name.is_empty() {
                Some(payee_name.clone())
            } else {
                self.extract_merchant(&description)
            };

            let tx = if let Some(m) = merchant {
                tx.with_merchant(m)
            } else {
                tx
            };

            transactions.push(tx);
        }

        Ok(transactions)
    }

    fn source_type(&self) -> SourceType {
        SourceType::Wise
    }
}

impl MerchantExtractor for WiseParser {
    fn extract_merchant(&self, description: &str) -> Option<String> {
        // Wise description patterns:
        // "Payment from Bloom Financial" → "Bloom Financial"
        // "Convert USD to MXN" → "Convert"
        // "Invoice payment" → "Invoice"

        if description.is_empty() {
            return None;
        }

        // Pattern 1: "Payment from X" → X
        if let Some(from_pos) = description.find("from ") {
            let merchant = description[from_pos + 5..].trim();
            if !merchant.is_empty() {
                return Some(merchant.to_string());
            }
        }

        // Pattern 2: "Payment to X" → X
        if let Some(to_pos) = description.find("to ") {
            let merchant = description[to_pos + 3..].trim();
            if !merchant.is_empty() {
                return Some(merchant.to_string());
            }
        }

        // Pattern 3: Take first word
        let first_word = description.split_whitespace().next()?;
        if first_word.len() > 2 {
            Some(first_word.to_string())
        } else {
            None
        }
    }
}

impl TypeClassifier for WiseParser {
    fn classify_type(&self, description: &str, amount: f64) -> String {
        let desc_lower = description.to_lowercase();

        // Wise is money transfer service
        // Incoming = INGRESO
        // Outgoing currency conversion = TRASPASO
        // Payments out = GASTO

        // Currency conversions
        if desc_lower.contains("convert") || desc_lower.contains("exchange") {
            return "TRASPASO".to_string();
        }

        // Incoming payments
        if amount > 0.0 || desc_lower.contains("payment from") || desc_lower.contains("received") {
            return "INGRESO".to_string();
        }

        // Outgoing payments
        if desc_lower.contains("payment to") || desc_lower.contains("invoice") {
            return "GASTO".to_string();
        }

        // Default: transfers
        "TRASPASO".to_string()
    }
}

/// Scotiabank Parser (Badge 11)
pub struct ScotiabankParser;

impl ScotiabankParser {
    pub fn new() -> Self {
        ScotiabankParser
    }
}

impl BankParser for ScotiabankParser {
    fn parse(&self, _file_path: &Path) -> Result<Vec<RawTransaction>> {
        // TODO: Implement in Badge 11
        Ok(Vec::new())
    }

    fn source_type(&self) -> SourceType {
        SourceType::Scotiabank
    }
}

impl MerchantExtractor for ScotiabankParser {
    fn extract_merchant(&self, _description: &str) -> Option<String> {
        // TODO: Implement in Badge 11
        None
    }
}

impl TypeClassifier for ScotiabankParser {
    fn classify_type(&self, _description: &str, _amount: f64) -> String {
        "GASTO".to_string()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_type_names() {
        assert_eq!(SourceType::BankOfAmerica.name(), "Bank of America");
        assert_eq!(SourceType::AppleCard.name(), "AppleCard");
        assert_eq!(SourceType::Stripe.name(), "Stripe");
        assert_eq!(SourceType::Wise.name(), "Wise");
        assert_eq!(SourceType::Scotiabank.name(), "Scotiabank");
    }

    #[test]
    fn test_source_type_codes() {
        assert_eq!(SourceType::BankOfAmerica.code(), "BofA");
        assert_eq!(SourceType::AppleCard.code(), "Apple");
        assert_eq!(SourceType::Stripe.code(), "Stripe");
        assert_eq!(SourceType::Wise.code(), "Wise");
        assert_eq!(SourceType::Scotiabank.code(), "Scotia");
    }

    #[test]
    fn test_detect_source_bofa() {
        let path = Path::new("bofa_march_2024.csv");
        let result = detect_source(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SourceType::BankOfAmerica);
    }

    #[test]
    fn test_detect_source_apple() {
        let path = Path::new("Apple Card Activity.csv");
        let result = detect_source(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SourceType::AppleCard);
    }

    #[test]
    fn test_detect_source_stripe() {
        let path = Path::new("stripe_january_2024.json");
        let result = detect_source(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SourceType::Stripe);
    }

    #[test]
    fn test_detect_source_wise() {
        let path = Path::new("wise_statement.csv");
        let result = detect_source(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SourceType::Wise);
    }

    #[test]
    fn test_detect_source_scotia() {
        let path = Path::new("scotia_feb_2024.csv");
        let result = detect_source(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SourceType::Scotiabank);
    }

    #[test]
    fn test_detect_source_unknown() {
        let path = Path::new("unknown_bank.csv");
        let result = detect_source(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_parser_bofa() {
        let parser = get_parser(SourceType::BankOfAmerica);
        assert_eq!(parser.source_type(), SourceType::BankOfAmerica);
    }

    #[test]
    fn test_get_parser_apple() {
        let parser = get_parser(SourceType::AppleCard);
        assert_eq!(parser.source_type(), SourceType::AppleCard);
    }

    #[test]
    fn test_raw_transaction_builder() {
        let tx = RawTransaction::new(
            "2024-03-20".to_string(),
            "STARBUCKS".to_string(),
            "-45.99".to_string(),
            SourceType::BankOfAmerica,
            "bofa_march.csv".to_string(),
            23,
            "03/20/2024,STARBUCKS,-45.99".to_string(),
        )
        .with_merchant("STARBUCKS".to_string())
        .with_category("Food".to_string())
        .with_confidence(0.95);

        assert_eq!(tx.date, "2024-03-20");
        assert_eq!(tx.merchant, Some("STARBUCKS".to_string()));
        assert_eq!(tx.category, Some("Food".to_string()));
        assert_eq!(tx.confidence, Some(0.95));
    }

    #[test]
    fn test_bofa_parser_parse_csv() {
        let parser = BofAParser::new();
        let path = Path::new("test_bofa.csv");
        let result = parser.parse(path);

        assert!(result.is_ok(), "Parser should successfully parse CSV");
        let txs = result.unwrap();
        assert_eq!(txs.len(), 3, "Should parse 3 transactions");

        // Check first transaction
        assert_eq!(txs[0].date, "12/31/2024");
        assert!(txs[0].description.contains("Stripe"));
        assert_eq!(txs[0].amount, "-$855.94");
        assert_eq!(txs[0].source_type, SourceType::BankOfAmerica);
    }

    #[test]
    fn test_bofa_extract_merchant_stripe() {
        let parser = BofAParser::new();
        let desc = "Stripe, Des:transfer, Id:st-n6u2j7l7r5l0";
        let merchant = parser.extract_merchant(desc);

        assert!(merchant.is_some());
        assert_eq!(merchant.unwrap(), "Stripe");
    }

    #[test]
    fn test_bofa_extract_merchant_wise() {
        let parser = BofAParser::new();
        let desc = "Wise Us Inc, Des:thera Pay, Id:thera Pay";
        let merchant = parser.extract_merchant(desc);

        assert!(merchant.is_some());
        assert_eq!(merchant.unwrap(), "Wise Us Inc");
    }

    #[test]
    fn test_bofa_classify_credit_card_payment() {
        let parser = BofAParser::new();
        let desc = "Bank of America Credit Card Bill Payment";
        let type_result = parser.classify_type(desc, -3047.57);

        assert_eq!(type_result, "PAGO_TARJETA");
    }

    #[test]
    fn test_bofa_classify_transfer() {
        let parser = BofAParser::new();
        let desc = "Stripe, Des:transfer, Id:st-n6u2j7l7r5l0";
        let type_result = parser.classify_type(desc, -855.94);

        assert_eq!(type_result, "TRASPASO");
    }

    #[test]
    fn test_bofa_classify_income() {
        let parser = BofAParser::new();
        let desc = "Wise Us Inc, Des:thera Pay, Id:thera Pay";
        let type_result = parser.classify_type(desc, 2000.0);

        assert_eq!(type_result, "INGRESO");
    }

    // ============================================================================
    // AppleCard Parser Tests (Badge 8)
    // ============================================================================

    #[test]
    fn test_apple_parser_parse_csv() {
        let parser = AppleCardParser::new();
        let path = Path::new("test_apple.csv");
        let result = parser.parse(path);

        assert!(result.is_ok(), "Parser should successfully parse CSV");
        let txs = result.unwrap();
        assert_eq!(txs.len(), 3, "Should parse 3 transactions");

        // Check first transaction
        assert_eq!(txs[0].date, "10/26/2024");
        assert!(txs[0].description.contains("UBER"));
        assert_eq!(txs[0].amount, "3.74");
        assert_eq!(txs[0].source_type, SourceType::AppleCard);
        assert_eq!(txs[0].merchant, Some("Uber Eats".to_string()));
        assert_eq!(txs[0].category, Some("Restaurants".to_string()));
    }

    #[test]
    fn test_apple_extract_merchant_uber() {
        let parser = AppleCardParser::new();
        let desc = "UBER *EATS MR TREUBLAAN 7 AMSTERDAM 1097 DP NH NLD";
        let merchant = parser.extract_merchant(desc);

        assert!(merchant.is_some());
        assert_eq!(merchant.unwrap(), "UBER *EATS");
    }

    #[test]
    fn test_apple_classify_payment() {
        let parser = AppleCardParser::new();
        let desc = "ACH DEPOSIT INTERNET TRANSFER FROM ACCOUNT ENDING IN 5226";
        let type_result = parser.classify_type(desc, -938.16);

        assert_eq!(type_result, "PAGO_TARJETA");
    }

    #[test]
    fn test_apple_classify_expense() {
        let parser = AppleCardParser::new();
        let desc = "UBER *EATS MR TREUBLAAN 7 AMSTERDAM";
        let type_result = parser.classify_type(desc, 3.74);

        assert_eq!(type_result, "GASTO");
    }

    // ============================================================================
    // Stripe Parser Tests (Badge 9)
    // ============================================================================

    #[test]
    fn test_stripe_parser_parse_json() {
        let parser = StripeParser::new();
        let path = Path::new("test_stripe.json");
        let result = parser.parse(path);

        assert!(result.is_ok(), "Parser should successfully parse JSON");
        let txs = result.unwrap();
        assert_eq!(txs.len(), 3, "Should parse 3 transactions");

        // Check first transaction
        assert_eq!(txs[0].date, "12/25/2024");
        assert!(txs[0].description.contains("Payment from eugenio Castro Garza"));
        assert_eq!(txs[0].amount, "2867.70");
        assert_eq!(txs[0].source_type, SourceType::Stripe);
        assert_eq!(txs[0].merchant, Some("eugenio Castro Garza".to_string()));
    }

    #[test]
    fn test_stripe_extract_merchant_payment_from() {
        let parser = StripeParser::new();
        let desc = "Payment from eugenio Castro Garza";
        let merchant = parser.extract_merchant(desc);

        assert!(merchant.is_some());
        assert_eq!(merchant.unwrap(), "eugenio Castro Garza");
    }

    #[test]
    fn test_stripe_classify_payout() {
        let parser = StripeParser::new();
        let desc = "Payment from eugenio Castro Garza (ID: txn_123)";
        let type_result = parser.classify_type(desc, 2867.70);

        assert_eq!(type_result, "INGRESO");
    }

    #[test]
    fn test_stripe_classify_refund() {
        let parser = StripeParser::new();
        let desc = "Refund for charge ch_123";
        let type_result = parser.classify_type(desc, -50.00);

        assert_eq!(type_result, "GASTO");
    }

    #[test]
    fn test_stripe_classify_fee() {
        let parser = StripeParser::new();
        let desc = "Stripe fee for transaction";
        let type_result = parser.classify_type(desc, -2.50);

        assert_eq!(type_result, "GASTO");
    }

    // ============================================================================
    // Wise Parser Tests (Badge 10)
    // ============================================================================

    #[test]
    fn test_wise_parser_parse_csv() {
        let parser = WiseParser::new();
        let path = Path::new("test_wise.csv");
        let result = parser.parse(path);

        assert!(result.is_ok(), "Parser should successfully parse CSV");
        let txs = result.unwrap();
        assert_eq!(txs.len(), 5, "Should parse 5 transactions");

        // Check first transaction (USD - no conversion)
        assert_eq!(txs[0].date, "12/31/2024");
        assert!(txs[0].description.contains("Bloom Financial"));
        assert_eq!(txs[0].amount, "2000.00");
        assert_eq!(txs[0].source_type, SourceType::Wise);
        assert_eq!(txs[0].merchant, Some("Bloom Financial Corp".to_string()));
    }

    #[test]
    fn test_wise_currency_conversion_eur_to_usd() {
        let parser = WiseParser::new();
        let path = Path::new("test_wise.csv");
        let result = parser.parse(path);

        assert!(result.is_ok());
        let txs = result.unwrap();

        // Third transaction: 500 EUR → USD
        // Exchange rate 0.93 means 1 USD = 0.93 EUR
        // So 500 EUR / 0.93 = ~537.63 USD
        assert!(txs[2].description.contains("EUR"));
        assert!(txs[2].description.contains("USD"));
        let amount: f64 = txs[2].amount.parse().unwrap();
        assert!((amount - 537.63).abs() < 1.0, "EUR conversion should be ~537.63 USD");
    }

    #[test]
    fn test_wise_currency_conversion_mxn_to_usd() {
        let parser = WiseParser::new();
        let path = Path::new("test_wise.csv");
        let result = parser.parse(path);

        assert!(result.is_ok());
        let txs = result.unwrap();

        // Fourth transaction: -41000 MXN → USD
        // Exchange rate 20.00 means 1 USD = 20 MXN
        // So 41000 MXN / 20 = 2050 USD
        assert!(txs[3].description.contains("MXN"));
        assert!(txs[3].description.contains("USD"));
        let amount: f64 = txs[3].amount.parse().unwrap();
        assert_eq!(amount, 2050.00, "MXN conversion should be exactly 2050 USD");
    }

    #[test]
    fn test_wise_extract_merchant_payment_from() {
        let parser = WiseParser::new();
        let desc = "Payment from Bloom Financial";
        let merchant = parser.extract_merchant(desc);

        assert!(merchant.is_some());
        assert_eq!(merchant.unwrap(), "Bloom Financial");
    }

    #[test]
    fn test_wise_classify_incoming() {
        let parser = WiseParser::new();
        let desc = "Payment from Bloom Financial";
        let type_result = parser.classify_type(desc, 2000.00);

        assert_eq!(type_result, "INGRESO");
    }

    #[test]
    fn test_wise_classify_currency_conversion() {
        let parser = WiseParser::new();
        let desc = "Convert USD to MXN";
        let type_result = parser.classify_type(desc, -2000.00);

        assert_eq!(type_result, "TRASPASO");
    }

    #[test]
    fn test_wise_classify_outgoing_payment() {
        let parser = WiseParser::new();
        let desc = "Payment to supplier";
        let type_result = parser.classify_type(desc, -500.00);

        assert_eq!(type_result, "GASTO");
    }
}

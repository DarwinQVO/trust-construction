# Badge 30: Rules as Data (Reify Decisions) 📐

**Status:** PLANNED
**Started:** TBD
**Rich Hickey Feedback:** "Don't hide decisions in code. Make them explicit as DATA."

---

## 🎯 Objetivo

Convertir reglas hard-coded en datos:
- **Classification rules** = JSON/CUE files, no código
- **Deduplication rules** = Configurables, versionados
- **Validation rules** = Datos, no predicados hard-coded
- **Audit trail** = Qué regla se usó cuándo

---

## 📊 Problema Actual

```rust
// AHORA: Reglas hard-coded en código
impl Deduplicator {
    fn fuzzy_match(&self, tx1: &Transaction, tx2: &Transaction) -> bool {
        let date_diff = (tx1.date - tx2.date).num_days().abs();
        let amount_diff = (tx1.amount - tx2.amount).abs();

        date_diff <= 1 &&      // ❌ Hard-coded: ±1 day
        amount_diff <= 0.50    // ❌ Hard-coded: ±$0.50
    }
}

impl Classifier {
    fn classify(&self, tx: &Transaction) -> String {
        if tx.merchant.contains("STARBUCKS") {  // ❌ Hard-coded rule
            return "Coffee".to_string();
        }
        // ...
    }
}
```

**Rich dice:** "No puedes cambiar estas reglas sin recompilar. No puedes A/B test. No puedes auditar qué regla se usó."

---

## ✅ Solución: Rules as Explicit Data

```rust
// DEDUPLICATION RULES (data, not code)
#[derive(Deserialize, Serialize)]
struct DeduplicationRule {
    name: String,
    version: String,
    date_tolerance_days: i64,
    amount_tolerance: f64,
    merchant_similarity_threshold: f64,
    confidence: f64,  // How confident is this rule
}

// Load from CUE/JSON
let rules: Vec<DeduplicationRule> = load_rules("rules/deduplication.cue")?;

// rules/deduplication.cue:
/*
rules: [
    {
        name: "exact_match"
        version: "1.0"
        date_tolerance_days: 0
        amount_tolerance: 0.00
        merchant_similarity_threshold: 1.0
        confidence: 0.99
    },
    {
        name: "fuzzy_match"
        version: "1.0"
        date_tolerance_days: 1
        amount_tolerance: 0.50
        merchant_similarity_threshold: 0.85
        confidence: 0.75
    },
    {
        name: "very_fuzzy"
        version: "1.0"
        date_tolerance_days: 3
        amount_tolerance: 2.00
        merchant_similarity_threshold: 0.70
        confidence: 0.50
    }
]
*/

// Apply rules (generic function)
fn find_duplicates(
    txs: &[Transaction],
    rules: &[DeduplicationRule]
) -> Vec<DuplicateCandidate> {
    let mut candidates = vec![];

    for rule in rules {
        for pair in all_pairs(txs) {
            if matches_rule(pair, rule) {
                candidates.push(DuplicateCandidate {
                    tx1: pair.0.id,
                    tx2: pair.1.id,
                    confidence: rule.confidence,
                    matched_by_rule: rule.name.clone(),
                    rule_version: rule.version.clone(),
                });
            }
        }
    }

    candidates
}

// CLASSIFICATION RULES (data, not code)
#[derive(Deserialize, Serialize)]
struct ClassificationRule {
    name: String,
    version: String,
    conditions: Vec<Condition>,
    category: String,
    confidence: f64,
}

#[derive(Deserialize, Serialize)]
enum Condition {
    MerchantContains(String),
    MerchantExactly(String),
    AmountGreaterThan(f64),
    AmountLessThan(f64),
    AmountBetween(f64, f64),
    DescriptionContains(String),
    And(Vec<Condition>),
    Or(Vec<Condition>),
}

// rules/classification.cue:
/*
rules: [
    {
        name: "starbucks_coffee"
        version: "2.0"
        conditions: [
            { MerchantContains: "STARBUCKS" }
        ]
        category: "Coffee"
        confidence: 0.95
    },
    {
        name: "large_transfer"
        version: "1.0"
        conditions: [
            { And: [
                { AmountGreaterThan: 1000.00 },
                { DescriptionContains: "TRANSFER" }
            ]}
        ]
        category: "Transfer"
        confidence: 0.90
    }
]
*/

// Evaluate conditions (interpreter)
fn evaluate_condition(tx: &Transaction, cond: &Condition) -> bool {
    match cond {
        Condition::MerchantContains(s) => tx.merchant.contains(s),
        Condition::AmountGreaterThan(x) => tx.amount > *x,
        Condition::And(conds) => conds.iter().all(|c| evaluate_condition(tx, c)),
        Condition::Or(conds) => conds.iter().any(|c| evaluate_condition(tx, c)),
        // ...
    }
}

// Apply classification rules
fn classify(tx: &Transaction, rules: &[ClassificationRule]) -> Vec<Classification> {
    rules.iter()
        .filter(|rule| {
            rule.conditions.iter().all(|c| evaluate_condition(tx, c))
        })
        .map(|rule| Classification {
            tx_id: tx.id,
            category: rule.category.clone(),
            confidence: rule.confidence,
            classified_by: format!("rule:{}:{}", rule.name, rule.version),
            // ...
        })
        .collect()
}
```

---

## 📋 Implementation Tasks

### Phase 1: Rule Types (2 hours)

- [ ] Create `src/rules/mod.rs`
- [ ] Define `DeduplicationRule` struct
- [ ] Define `ClassificationRule` struct
- [ ] Define `Condition` enum (interpreter)
- [ ] Tests: deserialize rules from JSON/CUE

### Phase 2: Rule Loading (2 hours)

- [ ] Create `rules/` directory
- [ ] `rules/deduplication.cue`
- [ ] `rules/classification.cue`
- [ ] Load rules on startup
- [ ] Validate rules (CUE validation)
- [ ] Tests: load, validate

### Phase 3: Rule Evaluation (3 hours)

- [ ] Implement `evaluate_condition()` interpreter
- [ ] Implement `matches_rule()` for dedup
- [ ] Implement `apply_rules()` generic
- [ ] Tests: rule evaluation

### Phase 4: Refactor Existing Logic (3 hours)

- [ ] Replace hard-coded dedup with rules
- [ ] Replace hard-coded classification with rules
- [ ] Store which rule was used (audit trail)
- [ ] Tests: same results, now configurable

### Phase 5: Rule Versioning (2 hours)

- [ ] Track rule version in decisions
- [ ] Can load multiple rule versions
- [ ] Migrate from old to new rules
- [ ] Tests: rule versioning, migration

---

## 🧪 Criterios de Éxito

```rust
#[test]
fn test_rules_are_data() {
    // Load rules from file (NOT code)
    let rules: Vec<DeduplicationRule> = load_rules("test-rules.cue").unwrap();

    assert_eq!(rules.len(), 3);
    assert_eq!(rules[0].name, "exact_match");
    assert_eq!(rules[1].date_tolerance_days, 1);

    // ✅ Rules are data, can be modified without recompiling
}

#[test]
fn test_multiple_rule_versions() {
    let v1_rules = load_rules("rules/dedup-v1.cue").unwrap();
    let v2_rules = load_rules("rules/dedup-v2.cue").unwrap();

    let txs = test_transactions();

    // Apply both versions
    let v1_results = find_duplicates(&txs, &v1_rules);
    let v2_results = find_duplicates(&txs, &v2_rules);

    // ✅ Can compare results from different rule versions
    assert!(v2_results.len() > v1_results.len());  // v2 is more lenient
}

#[test]
fn test_audit_trail_includes_rules() {
    let rules = load_rules("rules/classification.cue").unwrap();
    let tx = Transaction { merchant: "STARBUCKS", ... };

    let classifications = classify(&tx, &rules);

    // ✅ Know WHICH rule classified it
    assert_eq!(classifications[0].classified_by, "rule:starbucks_coffee:2.0");

    // ✅ Can audit: "This was classified by rule version 2.0"
}

#[test]
fn test_ab_testing_rules() {
    let rule_a = DeduplicationRule {
        name: "fuzzy_a",
        date_tolerance_days: 1,
        amount_tolerance: 0.50,
        confidence: 0.75,
        // ...
    };

    let rule_b = DeduplicationRule {
        name: "fuzzy_b",
        date_tolerance_days: 2,
        amount_tolerance: 1.00,
        confidence: 0.65,
        // ...
    };

    let txs = test_transactions();

    // Apply both rules
    let results_a = find_duplicates(&txs, &[rule_a]);
    let results_b = find_duplicates(&txs, &[rule_b]);

    // ✅ Compare which rule performs better
    println!("Rule A found {} duplicates", results_a.len());
    println!("Rule B found {} duplicates", results_b.len());
}

#[test]
fn test_condition_interpreter() {
    let tx = Transaction {
        merchant: "STARBUCKS",
        amount: 45.99,
        // ...
    };

    let cond = Condition::And(vec![
        Condition::MerchantContains("STARBUCKS".to_string()),
        Condition::AmountLessThan(50.0),
    ]);

    // ✅ Interpreter evaluates conditions
    assert!(evaluate_condition(&tx, &cond));
}
```

---

## 🎁 Benefits

1. **No Recompile** - Change rules without rebuilding
2. **A/B Testing** - Test different rules on same data
3. **Audit Trail** - Know which rule version was used
4. **Domain Experts** - Non-programmers can modify rules
5. **Versioning** - Track rule evolution over time
6. **Testing** - Test rules as data, not code

---

## 📊 Metrics

- [ ] 0 hard-coded classification rules
- [ ] 0 hard-coded deduplication thresholds
- [ ] All rules loaded from CUE files
- [ ] Tests: 25+ rules-as-data tests

---

## 🔧 Advanced: CUE Validation

```cue
// rules/schema.cue
#DeduplicationRule: {
    name: string
    version: string
    date_tolerance_days: int & >=0 & <=7
    amount_tolerance: number & >=0 & <=100
    merchant_similarity_threshold: number & >=0 & <=1
    confidence: number & >0 & <=1
}

rules: [...#DeduplicationRule]

// ✅ CUE validates at load time:
// - date_tolerance must be 0-7
// - confidence must be 0-1
// - all required fields present
```

---

**Previous:** Badge 29 - Schema Refinement
**Next:** Production deployment with all Rich Hickey principles

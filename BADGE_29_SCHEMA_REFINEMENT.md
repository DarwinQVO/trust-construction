# Badge 29: Schema Refinement (Decomplecting) 🎨

**Status:** PLANNED
**Started:** TBD
**Rich Hickey Feedback:** "Separate shape, selection, and qualification. Confidence is metadata, not transaction data."

---

## 🎯 Objetivo

Decomplectar (separar) concerns mezclados:
1. **Shape** (qué campos PUEDEN existir) vs **Selection** (qué campos NECESITO)
2. **Facts** (la transacción ocurrió) vs **Inference** (confianza en clasificación)
3. **Usage contexts** vs **Data model**

---

## 📊 Problemas Actuales

### Problema 1: Contexts mezclan concerns

```rust
// AHORA: SchemaContext mezcla 3 cosas diferentes
enum SchemaContext {
    UI,          // ← SELECTION (qué mostrar)
    Audit,       // ← METADATA (propiedades)
    Import,      // ← PROVENANCE (de dónde)
    MLTraining,  // ← QUALIFICATION (¿es apto?)
}
```

### Problema 2: Confidence en schema core

```rust
// AHORA: Confidence está en Transaction
struct Transaction {
    amount: f64,               // ← Hecho
    confidence_score: f64,     // ← Inferencia (¡MEZCLADO!)
}
```

**Rich dice:** "La transacción OCURRIÓ (hecho). Tu confianza es SEPARADA (inferencia)."

---

## ✅ Solución 1: Separate Shape/Selection/Qualification

```rust
// SHAPE: What CAN exist (complete schema)
struct Transaction {
    // Identity
    id: UUID,
    version: i64,

    // Business facts
    date: Date,
    amount: Decimal,
    merchant: String,
    description: String,

    // Temporal
    valid_from: DateTime,
    valid_until: Option<DateTime>,

    // Provenance
    source_file: String,
    imported_at: DateTime,
    imported_by: String,

    // ALL possible fields (the complete shape)
}

// SELECTION: What's needed in each context (subsets)
mod selections {
    pub fn ui_fields() -> Vec<Field> {
        vec![
            Field::Date,
            Field::Merchant,
            Field::Amount,
            Field::Category,  // From classification
        ]
    }

    pub fn audit_fields() -> Vec<Field> {
        vec![
            Field::Id,
            Field::Version,
            Field::SourceFile,
            Field::ImportedAt,
            Field::ImportedBy,
            Field::ValidFrom,
            Field::ValidUntil,
        ]
    }

    pub fn export_fields() -> Vec<Field> {
        vec![
            Field::Date,
            Field::Merchant,
            Field::Amount,
            Field::Description,
        ]
    }
}

// QUALIFICATION: What qualifies for specific uses
mod qualifications {
    pub fn ml_ready(tx: &Transaction, classification: &Classification) -> bool {
        // Qualified for ML training if:
        classification.verified &&                    // Human verified
        classification.confidence > 0.8 &&            // High confidence
        !tx.is_duplicate() &&                         // Not a duplicate
        tx.imported_at < Utc::now() - Duration::days(30)  // Aged data
    }

    pub fn audit_required(tx: &Transaction) -> bool {
        tx.amount.abs() > Decimal::from(10000) ||     // Large amount
        tx.merchant.contains("International")          // Cross-border
    }

    pub fn export_ready(tx: &Transaction) -> bool {
        tx.reconciled &&                               // Reconciled
        !tx.is_pending_review()                        // No pending issues
    }
}
```

---

## ✅ Solución 2: Separate Facts from Inference

```rust
// FACTS: Transaction (what happened)
struct Transaction {
    id: UUID,
    date: Date,
    amount: Decimal,
    merchant: String,
    description: String,

    // NO confidence, NO category (those are inferences)
}

// INFERENCE: Classification (what we think it is)
struct Classification {
    tx_id: UUID,
    category: String,
    confidence: f64,              // ← Confidence belongs HERE
    classified_at: DateTime,
    classified_by: String,        // "system:v1.0" or "user:darwin"
    verified: bool,               // Human verified?
    verified_by: Option<String>,
    verified_at: Option<DateTime>,
}

// INFERENCE: Deduplication decision
struct DuplicationDecision {
    tx1_id: UUID,
    tx2_id: UUID,
    is_duplicate: bool,
    confidence: f64,              // ← Confidence for dedup
    decided_by: String,
    decided_at: DateTime,
    reason: String,
}

// Now you can have MULTIPLE classifications!
let classifications = vec![
    Classification {
        tx_id: uuid,
        category: "Food",
        confidence: 0.85,
        classified_by: "system:v1.0",
        verified: false,
        // ...
    },
    Classification {
        tx_id: uuid,
        category: "Dining",          // Human reclassified
        confidence: 1.0,
        classified_by: "user:darwin",
        verified: true,
        // ...
    },
];
```

---

## 📋 Implementation Tasks

### Phase 1: Separate Contexts (2 hours)

- [ ] Create `src/selections.rs`
- [ ] Define `ui_fields()`, `audit_fields()`, etc.
- [ ] Create `src/qualifications.rs`
- [ ] Define `ml_ready()`, `export_ready()`, etc.
- [ ] Remove `SchemaContext` enum
- [ ] Tests: selections return correct subsets

### Phase 2: Extract Inference Structs (3 hours)

- [ ] Create `Classification` struct
- [ ] Create `DuplicationDecision` struct
- [ ] Remove `confidence_score` from Transaction
- [ ] Remove `category` from Transaction core
- [ ] Tests: facts separate from inference

### Phase 3: Multi-Classification Support (2 hours)

- [ ] `HashMap<UUID, Vec<Classification>>` - multiple per tx
- [ ] Get current classification (latest verified, or highest confidence)
- [ ] Track classification history
- [ ] Tests: multiple classifications, history

### Phase 4: Refactor Queries (2 hours)

- [ ] Update UI to use selections
- [ ] Update audit to use selections
- [ ] Qualification predicates in filters
- [ ] Tests: queries work with new structure

### Phase 5: Schema Evolution (1 hour)

- [ ] Document accretion-only rules
- [ ] Version schema files
- [ ] Schema version in serialized data
- [ ] Tests: old data loads in new schema

---

## 🧪 Criterios de Éxito

```rust
#[test]
fn test_shape_vs_selection() {
    let tx = Transaction::new(...);  // Has ALL fields (shape)

    // Selection: Only what UI needs
    let ui_data = select_fields(&tx, &ui_fields());
    assert_eq!(ui_data.len(), 4);  // date, merchant, amount, category

    // Selection: Only what audit needs
    let audit_data = select_fields(&tx, &audit_fields());
    assert_eq!(audit_data.len(), 7);  // id, version, provenance, temporal
}

#[test]
fn test_qualification_predicates() {
    let tx = Transaction::new(...);
    let classification = Classification { confidence: 0.9, verified: true, ... };

    // ✅ Qualifies for ML training
    assert!(ml_ready(&tx, &classification));

    // Different transaction
    let tx2 = Transaction { amount: 50000.0, ... };

    // ✅ Requires audit (large amount)
    assert!(audit_required(&tx2));
}

#[test]
fn test_facts_separate_from_inference() {
    let tx = Transaction {
        id: uuid,
        amount: 42.50,
        merchant: "Starbucks",
        // NO confidence, NO category
    };

    // Classifications are separate
    let system_classification = Classification {
        tx_id: uuid,
        category: "Food",
        confidence: 0.85,
        classified_by: "system:v1.0",
        verified: false,
    };

    let human_classification = Classification {
        tx_id: uuid,
        category: "Dining",
        confidence: 1.0,
        classified_by: "user:darwin",
        verified: true,
    };

    // ✅ Multiple classifications coexist
    let all = vec![system_classification, human_classification];

    // ✅ Get current (highest confidence verified)
    let current = get_current_classification(&all);
    assert_eq!(current.category, "Dining");
}
```

---

## 🎁 Benefits

1. **Clear Separation** - Shape, selection, qualification are distinct
2. **Multiple Inferences** - Track system + human classifications
3. **Flexible Contexts** - Easy to add new selections
4. **Testing** - Test qualifications as pure predicates
5. **Schema Evolution** - Accretion-only, no breakage

---

## 📊 Metrics

- [ ] 0 inference fields in Transaction core
- [ ] Multiple classifications per transaction supported
- [ ] Selections defined as data (not code)
- [ ] Tests: 20+ schema refinement tests

---

**Previous:** Badge 28 - Value Store
**Next:** Badge 30 - Rules as Data

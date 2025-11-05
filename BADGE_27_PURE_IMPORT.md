# Badge 27: Pure Import (Facts vs Inference) 📥

**Status:** PLANNED
**Started:** TBD
**Rich Hickey Feedback:** "Import is about FACTS. 'Are duplicates' is INFERENCE. Don't conflate them."

---

## 🎯 Objetivo

Separar completamente:
- **Import** = Solo hechos ("esta transacción llegó de este CSV en este timestamp")
- **Analysis** = Inferencias (deduplication, classification, validation)
- **Decision** = Marcado explícito (humano o sistema aprueba)

---

## 📊 Problema Actual

```rust
// AHORA: Import decide duplicados en el momento
fn import_csv(file: &Path) -> Result<Vec<Transaction>> {
    let raw = parse_csv(file)?;
    let deduplicated = deduplicate(&raw);  // ❌ Mezclando concerns
    Ok(deduplicated)
}
```

**Rich dice:** "Esto mezcla 2 cosas. Import debe ser PURO - solo transformar external → internal."

---

## ✅ Solución: Three-Stage Pipeline

```rust
// STAGE 1: Pure Import (NO analysis, just facts)
fn import_transactions(file: &Path) -> Result<Vec<ImportedTransaction>> {
    let raw = parse_csv(file)?;

    raw.into_iter().map(|r| ImportedTransaction {
        // FACTS only
        id: UUID::new(),
        data: r,
        source_file: file.to_string(),
        imported_at: Utc::now(),
        imported_by: "system",

        // NO confidence, NO dedup, NO classification
    }).collect()
}

// STAGE 2: Analysis (separate process)
fn analyze_for_duplicates(
    imported: &[ImportedTransaction],
    existing: &[Transaction],
) -> Vec<DuplicateCandidate> {
    // Analyze, don't decide
    find_potential_duplicates(imported, existing)
        .into_iter()
        .map(|(tx1, tx2, similarity)| DuplicateCandidate {
            tx1_id: tx1.id,
            tx2_id: tx2.id,
            similarity_score: similarity,
            reason: format!("Amount match: {:.2}, Date diff: {} days", ...),
            detected_at: Utc::now(),
        })
        .collect()
}

// STAGE 3: Decision (explicit marking)
fn mark_as_duplicate(
    tx1_id: UUID,
    tx2_id: UUID,
    decided_by: &str,  // "system" or "user:darwin"
    reason: &str,
) -> Event {
    Event::DuplicateMarked {
        tx1_id,
        tx2_id,
        decided_by: decided_by.to_string(),
        reason: reason.to_string(),
        decided_at: Utc::now(),
    }
}
```

---

## 📋 Implementation Tasks

### Phase 1: Pure Import Types (2 hours)

- [ ] Create `ImportedTransaction` struct (facts only)
- [ ] Remove dedup/classification from import pipeline
- [ ] `import_csv()` → pure transformation
- [ ] Tests: import returns ALL rows, no filtering

### Phase 2: Separate Analysis (3 hours)

- [ ] Create `DuplicateCandidate` struct
- [ ] Implement `analyze_for_duplicates()` - returns candidates, doesn't decide
- [ ] Implement `analyze_for_classification()` - suggests, doesn't apply
- [ ] Tests: analysis finds candidates, doesn't modify data

### Phase 3: Explicit Decisions (2 hours)

- [ ] Create `Decision` enum (DuplicateConfirmed, ClassificationApproved, etc.)
- [ ] Implement `apply_decision()` - creates events
- [ ] Store decisions as events
- [ ] Tests: decisions are explicit, trackable

### Phase 4: Refactor Pipeline (2 hours)

- [ ] Update main import flow
- [ ] Import → Store ALL
- [ ] Analyze → Detect candidates
- [ ] Review → User/system decides
- [ ] Tests: end-to-end

---

## 🧪 Criterios de Éxito

```rust
#[test]
fn test_import_stores_everything() {
    // CSV with 10 rows, 2 are duplicates
    let imported = import_csv("test.csv").unwrap();

    // ✅ ALL 10 rows imported (no filtering)
    assert_eq!(imported.len(), 10);

    // ✅ Each has provenance
    for tx in &imported {
        assert_eq!(tx.source_file, "test.csv");
        assert!(tx.imported_at > Utc::now() - Duration::seconds(5));
    }
}

#[test]
fn test_analysis_suggests_not_decides() {
    let imported = import_csv("test.csv").unwrap();
    let existing = load_existing_transactions();

    let candidates = analyze_for_duplicates(&imported, &existing);

    // ✅ Found duplicate candidates
    assert_eq!(candidates.len(), 2);

    // ✅ Candidates have similarity scores
    assert!(candidates[0].similarity_score > 0.9);

    // ✅ Original data unchanged
    assert_eq!(imported.len(), 10);  // Still all 10
}

#[test]
fn test_decisions_are_explicit() {
    let candidates = analyze_for_duplicates(&imported, &existing);

    // Human reviews and decides
    let decision = mark_as_duplicate(
        candidates[0].tx1_id,
        candidates[0].tx2_id,
        "user:darwin",
        "Confirmed: same merchant, amount, date",
    );

    // ✅ Decision is an event
    assert!(matches!(decision, Event::DuplicateMarked { .. }));

    // ✅ Decision has provenance
    assert_eq!(decision.decided_by, "user:darwin");
}
```

---

## 🎁 Benefits

1. **Pure Import** - Can re-import same file safely
2. **Separate Analysis** - Can run different algorithms without re-importing
3. **Explicit Decisions** - Know WHO decided WHAT and WHY
4. **Audit Trail** - See evolution of understanding
5. **Testing** - Test import, analysis, decisions independently

---

## 📊 Metrics

- [ ] 100% of imports are pure (no filtering)
- [ ] 0 analysis in import path
- [ ] All decisions are explicit events
- [ ] Tests: 20+ pure import tests

---

**Previous:** Badge 26 - Event Sourcing
**Next:** Badge 28 - Value Store + Index Separation

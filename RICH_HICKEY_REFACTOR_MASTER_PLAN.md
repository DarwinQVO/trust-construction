# 🎯 Rich Hickey Refactor - Master Plan

**Version:** 1.0
**Created:** 2025-11-04
**Status:** READY TO EXECUTE

---

## 📋 Executive Summary

Este plan implementa TODOS los 10 puntos del feedback de Rich Hickey sobre el sistema Trust Construction.

**Problema:** Filosofía correcta, herramientas equivocadas
**Solución:** 5 nuevos badges que transforman el sistema completo

---

## 🎖️ Badges Overview

| Badge | Título | Duración | Impacto | Status |
|-------|--------|----------|---------|--------|
| **26** | Event Sourcing Foundation | 12h | 🔴 CRÍTICO | PLANNED |
| **27** | Pure Import (Facts vs Inference) | 10h | 🔴 CRÍTICO | PLANNED |
| **28** | Value Store + Index Separation | 12h | 🔴 CRÍTICO | PLANNED |
| **29** | Schema Refinement (Decomplecting) | 10h | 🟡 ALTO | PLANNED |
| **30** | Rules as Data | 12h | 🟡 ALTO | PLANNED |

**Total:** ~56 horas (7-8 días de trabajo efectivo)

---

## 🗺️ Roadmap

### Week 1: Foundation (Badges 26-27)

**Badge 26: Event Sourcing** (Days 1-2)
- Events como fuente primaria de verdad
- Estado actual = fold de eventos
- Event Store inmutable

**Badge 27: Pure Import** (Days 2-3)
- Import solo hechos
- Analysis separado
- Decisiones explícitas

**Checkpoint:** Sistema tiene event sourcing y import puro funcionando

---

### Week 2: Storage Architecture (Badge 28)

**Badge 28: Value Store + Index** (Days 4-5)
- Value store inmutable (file-based)
- SQLite solo como índice
- Rebuild capability

**Checkpoint:** 0 UPDATE, 0 DELETE en todo el sistema

---

### Week 3: Refinement (Badges 29-30)

**Badge 29: Schema Refinement** (Day 6)
- Separate shape/selection/qualification
- Confidence → Classification struct
- Multi-classification support

**Badge 30: Rules as Data** (Days 7-8)
- Deduplication rules en CUE
- Classification rules en CUE
- Rule versioning

**Checkpoint:** Sistema 100% data-driven, zero hard-coded rules

---

## 📊 Before vs After

### Before (Badge 25)

```
✅ Filosofía: Inmutabilidad, temporal design, identity/value
❌ Herramientas: SQLite mutable, rules hard-coded
❌ Architecture: Estado es verdad, eventos secundarios
❌ Import: Mezcla hechos con inferencia
```

**Score:** 60% Rich Hickey Compliant

### After (Badge 30)

```
✅ Filosofía: Inmutabilidad, temporal design, identity/value
✅ Herramientas: Event Store, Value Store, CUE rules
✅ Architecture: Eventos son verdad, estado derivado
✅ Import: Hechos puros, análisis separado
✅ Schema: Shape/Selection/Qualification separados
✅ Rules: Todo configurable, versionado, auditable
```

**Score:** 100% Rich Hickey Compliant

---

## 🔧 Technical Architecture (After Badge 30)

```
┌─────────────────────────────────────────────────────────┐
│                    APPLICATION LAYER                     │
│  (UI, CLI, API - Imperative Shell)                      │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│              FUNCTIONAL CORE (Pure Logic)                │
│  - Projections (fold events → state)                    │
│  - Rules interpreter (evaluate conditions)               │
│  - Qualifications (predicates)                          │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│                    EVENT STORE                           │
│  (Append-only, immutable, source of truth)              │
│                                                          │
│  Events:                                                 │
│  - TransactionImported                                   │
│  - BankRegistered, BankUpdated                          │
│  - DuplicateDetected, DuplicateMarked                   │
│  - Classified, ClassificationVerified                   │
└────────────────┬────────────────────────────────────────┘
                 │
        ┌────────┴────────┐
        │                 │
┌───────▼──────┐  ┌───────▼──────┐
│ VALUE STORE  │  │    INDEX     │
│ (Immutable)  │  │ (Ephemeral)  │
│              │  │              │
│ Files:       │  │ SQLite:      │
│ - Txs        │  │ - by_date    │
│ - Events     │  │ - by_merchant│
│ - Entities   │  │ - by_amount  │
│              │  │              │
│ Hash-based   │  │ Rebuildable  │
│ Content-addr │  │ from values  │
└──────────────┘  └──────────────┘

┌─────────────────────────────────────────────────────────┐
│                    RULES (CUE Files)                     │
│  - deduplication.cue                                     │
│  - classification.cue                                    │
│  - validation.cue                                        │
│                                                          │
│  Versioned, Validated, Data-driven                      │
└─────────────────────────────────────────────────────────┘
```

---

## 🎯 Success Criteria (Badge 30 Complete)

### Code Metrics

- [ ] **0** hard-coded classification rules
- [ ] **0** hard-coded deduplication thresholds
- [ ] **0** UPDATE statements in entire codebase
- [ ] **0** DELETE statements in entire codebase
- [ ] **100%** of state changes are events
- [ ] **250+** tests passing (184 now + 66 new)

### Architecture Metrics

- [ ] Event Store is source of truth
- [ ] Current state = projection from events
- [ ] Import is pure (no analysis)
- [ ] Rules loaded from CUE files
- [ ] Index rebuildable from value store in <5s

### Philosophical Compliance

- [ ] ✅ Immutability (via event sourcing)
- [ ] ✅ Time as explicit dimension (via events)
- [ ] ✅ Identity/Value separation (via UUIDs + versions)
- [ ] ✅ Facts separate from inference (via Classifications)
- [ ] ✅ Decisions as data (via Rules as CUE)
- [ ] ✅ Accretion-only schema (no breakage)
- [ ] ✅ Values are forever, indexes ephemeral

---

## 📚 Documentation Structure

```
trust-construction/
├── BADGE_26_EVENT_SOURCING.md
├── BADGE_27_PURE_IMPORT.md
├── BADGE_28_VALUE_STORE.md
├── BADGE_29_SCHEMA_REFINEMENT.md
├── BADGE_30_RULES_AS_DATA.md
├── RICH_HICKEY_REFACTOR_MASTER_PLAN.md (this file)
│
├── src/
│   ├── events.rs              # Badge 26
│   ├── event_store.rs         # Badge 26
│   ├── projections.rs         # Badge 26
│   ├── import/
│   │   ├── pure_import.rs     # Badge 27
│   │   ├── analysis.rs        # Badge 27
│   │   └── decisions.rs       # Badge 27
│   ├── storage/
│   │   ├── value_store.rs     # Badge 28
│   │   └── index.rs           # Badge 28
│   ├── schema/
│   │   ├── selections.rs      # Badge 29
│   │   ├── qualifications.rs  # Badge 29
│   │   └── classification.rs  # Badge 29
│   └── rules/
│       ├── mod.rs             # Badge 30
│       ├── interpreter.rs     # Badge 30
│       └── loader.rs          # Badge 30
│
└── rules/
    ├── deduplication.cue
    ├── classification.cue
    └── validation.cue
```

---

## 🚀 Execution Strategy

### Option A: Sequential (Safe)
Implement badges 26→27→28→29→30 in order. Checkpoint after each.

**Pros:** Lower risk, tests always passing
**Cons:** Slower, 8 weeks total

### Option B: Parallel Streams (Fast)
- Stream 1: Badges 26 + 27 (Event + Import)
- Stream 2: Badge 28 (Value Store)
- Stream 3: Badges 29 + 30 (Schema + Rules)

**Pros:** Faster, 3-4 weeks total
**Cons:** Higher complexity, integration challenges

### Option C: Incremental (Hybrid) ⭐ RECOMMENDED
- Week 1: Badge 26 (Foundation)
- Week 2: Badges 27 + 28 (parallel)
- Week 3: Badges 29 + 30 (parallel)

**Pros:** Balanced speed + safety
**Cons:** None significant

---

## 🎓 Learning Outcomes

By Badge 30 you will have:

1. ✅ Implemented true Event Sourcing
2. ✅ Built content-addressable value store
3. ✅ Separated facts from inference
4. ✅ Made all decisions explicit as data
5. ✅ Achieved 100% Rich Hickey compliance
6. ✅ Built a production-grade financial system

**This is portfolio-grade architecture.**

---

## 📞 Rich Hickey Would Say...

> "Now you're not fighting your tools. The substrate WANTS to be immutable. Events are facts. Values are permanent. Rules are data. This is how you build systems that last decades, not months."

---

## 🎉 Badge 30 Completion = Gold Achievement

```
🏆 TRUST CONSTRUCTION SYSTEM
🥉 Bronze: Badges 1-5   (Foundation - UI working)
🥈 Silver: Badges 6-15  (Production Pipeline)
🥇 Gold:   Badges 16-25 (Trust Construction)
💎 Diamond: Badges 26-30 (Rich Hickey Compliant)
```

**Status after Badge 30:** 💎 **DIAMOND ACHIEVEMENT**

---

**Ready to execute?**
Start with: `BADGE_26_EVENT_SOURCING.md` → Phase 1

**Expected completion:** 7-8 days of focused work
**Result:** Production-grade, Rich Hickey-compliant financial system

---

**"Simple Made Easy. Time Made Explicit. Decisions Made Data."**
— Rich Hickey Philosophy, Fully Implemented

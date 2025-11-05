# Badge 26: Event Sourcing Foundation 🎯

**Status:** IN PROGRESS
**Started:** 2025-11-04
**Rich Hickey Feedback:** "Fully Embrace Event Sourcing - Events are the PRIMARY source of truth"

---

## 🎯 Objetivo

Implementar Event Sourcing puro donde:
- **Events** son la fuente primaria de verdad (no el estado actual)
- **Current state** = fold de todos los eventos
- **Audit trail** completo y reproducible
- **Time travel** gratis
- **Multiple projections** posibles

---

## 📊 Problema Actual

```rust
// AHORA: Estado es la fuente de verdad
struct BankRegistry {
    versions: Vec<Bank>,  // ← Estado actual
}

// Events solo para audit
struct AuditLog {
    events: Vec<Event>,  // ← Secundario
}
```

**Rich dice:** "Events deberían ser primarios, no secundarios"

---

## ✅ Solución: Event-First Architecture

```rust
// Event Store = Fuente primaria de verdad
enum Event {
    // Entity Events
    BankRegistered { id: UUID, data: Bank, at: DateTime },
    BankUpdated { id: UUID, changes: BankChanges, at: DateTime },

    // Transaction Events
    TransactionImported { id: UUID, data: Transaction, source: Source, at: DateTime },
    TransactionClassified { id: UUID, category: String, confidence: f64, by: String, at: DateTime },

    // Analysis Events
    DuplicateDetected { tx1: UUID, tx2: UUID, confidence: f64, reason: String, at: DateTime },
    DuplicateMarked { tx1: UUID, tx2: UUID, confirmed_by: String, at: DateTime },
}

// Event Store (append-only, immutable)
struct EventStore {
    events: Vec<Event>,  // ← FUENTE DE VERDAD
}

impl EventStore {
    fn append(&mut self, event: Event) {
        self.events.push(event);
        // Never delete, never modify
    }

    fn all_events(&self) -> &[Event] {
        &self.events
    }

    fn events_since(&self, timestamp: DateTime) -> Vec<Event> {
        self.events.iter()
            .filter(|e| e.timestamp() > timestamp)
            .cloned()
            .collect()
    }
}

// Estado actual = fold de eventos
fn current_banks(events: &[Event]) -> BankRegistry {
    events.iter().fold(BankRegistry::new(), |mut registry, event| {
        match event {
            Event::BankRegistered { data, .. } => {
                registry.register(data.clone());
            }
            Event::BankUpdated { id, changes, .. } => {
                registry.apply_changes(id, changes);
            }
            _ => {}
        }
        registry
    })
}
```

---

## 📋 Implementation Tasks

### Phase 1: Event Types (2 hours)

- [ ] Create `src/events.rs`
- [ ] Define `Event` enum con todos los tipos de eventos
- [ ] Implement `Event::timestamp()`, `Event::id()` helpers
- [ ] Tests: crear eventos, verificar fields

### Phase 2: Event Store (2 hours)

- [ ] Create `EventStore` struct
- [ ] Implement `append()` - append-only
- [ ] Implement `all_events()`, `events_since()`, `events_by_entity()`
- [ ] Implement persistence (SQLite con append-only table)
- [ ] Tests: append, query, persistence

### Phase 3: State Projections (3 hours)

- [ ] Implement `project_banks(events) -> BankRegistry`
- [ ] Implement `project_transactions(events) -> TransactionLedger`
- [ ] Implement `project_classifications(events) -> HashMap<UUID, Classification>`
- [ ] Tests: fold events, verify state matches

### Phase 4: Refactor Entities (3 hours)

- [ ] BankRegistry usa eventos internamente
- [ ] `register()` → `append(Event::BankRegistered)`
- [ ] `update()` → `append(Event::BankUpdated)`
- [ ] Tests: entities emit events

### Phase 5: Integration (2 hours)

- [ ] Main app usa EventStore
- [ ] Load events on startup
- [ ] Project current state
- [ ] Tests: end-to-end

---

## 🧪 Criterios de Éxito

```rust
#[test]
fn test_events_are_source_of_truth() {
    let mut store = EventStore::new();

    // Event 1: Register bank
    store.append(Event::BankRegistered {
        id: uuid1,
        data: Bank::new("Test Bank", "US", BankType::Checking),
        at: t1,
    });

    // Event 2: Update bank
    store.append(Event::BankUpdated {
        id: uuid1,
        changes: BankChanges { country: Some("CA") },
        at: t2,
    });

    // Project current state
    let registry = project_banks(&store.all_events());

    // ✅ State reflects events
    let bank = registry.find_by_id(&uuid1).unwrap();
    assert_eq!(bank.country, "CA");

    // ✅ Can replay with different logic
    let registry_v2 = project_banks_v2(&store.all_events());

    // ✅ Can query at any point in time
    let registry_at_t1 = project_banks(&store.events_until(t1));
    assert_eq!(registry_at_t1.find_by_id(&uuid1).unwrap().country, "US");
}
```

---

## 🎁 Benefits

1. **Complete Audit Trail** - Every change is an event
2. **Time Travel** - Project state at ANY point in time
3. **Reproducibility** - Replay events with new logic
4. **Multiple Views** - Different projections for different uses
5. **Testing** - Test projections with fixtures, no DB needed
6. **Debugging** - See EXACTLY what happened and when

---

## 📊 Metrics

- [ ] 100% of state changes are events
- [ ] 0 direct state mutations (all via events)
- [ ] Tests: 30+ event sourcing tests
- [ ] Can project state from empty to current in <100ms

---

**Next Badge:** Badge 27 - Pure Import (Separate Facts from Inference)

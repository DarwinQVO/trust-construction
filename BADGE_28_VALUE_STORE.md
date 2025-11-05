# Badge 28: Value Store + Index Separation 💾

**Status:** PLANNED
**Started:** TBD
**Rich Hickey Feedback:** "SQLite becomes just an index that can be rebuilt from the value store"

---

## 🎯 Objetivo

Separar:
- **Value Store** = Inmutable, append-only, valores permanentes
- **Index** = Efímero, reconstruible, solo para queries rápidas

---

## 📊 Problema Actual

```rust
// AHORA: SQLite ES el value store (mutable)
db.execute("UPDATE transactions SET category = ? WHERE id = ?");  // ❌ Mutation
db.execute("DELETE FROM transactions WHERE id = ?");              // ❌ Deletion
```

**Rich dice:** "Estás usando una base de datos MUTABLE para simular inmutabilidad. El substrate juega en tu contra."

---

## ✅ Solución: Datomic-Style Architecture

```rust
// VALUE STORE (immutable, append-only)
trait ValueStore {
    /// Store a value, return content-addressable hash
    fn put(&self, value: &Transaction) -> Hash;

    /// Retrieve value by hash
    fn get(&self, hash: &Hash) -> Option<Transaction>;

    /// Store event (events are also values)
    fn put_event(&self, event: &Event) -> Hash;

    /// Get all events (for replay)
    fn all_events(&self) -> Vec<Event>;
}

// Implementation: File-based (production: S3, IPFS, etc.)
struct FileValueStore {
    base_path: PathBuf,  // e.g., ~/.trust-construction/values/
}

impl FileValueStore {
    fn put(&self, value: &Transaction) -> Hash {
        // 1. Serialize to JSON/EDN
        let json = serde_json::to_string(value)?;

        // 2. Hash content (content-addressable)
        let hash = sha256(&json);

        // 3. Write to file (if doesn't exist)
        let path = self.base_path.join(format!("{}.json", hash));
        if !path.exists() {
            fs::write(path, json)?;
        }

        hash
    }
}

// INDEX (ephemeral, can be rebuilt)
trait Index {
    /// Query by date
    fn by_date(&self, date: Date) -> Vec<Hash>;

    /// Query by merchant
    fn by_merchant(&self, merchant: &str) -> Vec<Hash>;

    /// Query by amount range
    fn by_amount_range(&self, min: f64, max: f64) -> Vec<Hash>;

    /// Rebuild from value store
    fn rebuild(&mut self, values: &dyn ValueStore);
}

// Implementation: SQLite as INDEX only
struct SqliteIndex {
    conn: Connection,
}

impl SqliteIndex {
    fn schema() -> &'static str {
        r#"
        CREATE TABLE IF NOT EXISTS tx_index (
            hash TEXT PRIMARY KEY,
            date TEXT,
            merchant TEXT,
            amount REAL,
            -- indexes for fast queries
            indexed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX idx_date ON tx_index(date);
        CREATE INDEX idx_merchant ON tx_index(merchant);
        CREATE INDEX idx_amount ON tx_index(amount);
        "#
    }

    fn rebuild(&mut self, values: &dyn ValueStore) {
        // 1. Drop all indexes
        self.conn.execute("DELETE FROM tx_index", []).unwrap();

        // 2. Replay all events to rebuild
        for event in values.all_events() {
            match event {
                Event::TransactionImported { id, data, .. } => {
                    let hash = values.put(data);
                    self.index_transaction(&hash, data);
                }
                _ => {}
            }
        }
    }
}
```

---

## 📋 Implementation Tasks

### Phase 1: Value Store Trait (2 hours)

- [ ] Create `src/value_store.rs`
- [ ] Define `ValueStore` trait
- [ ] Implement `FileValueStore`
- [ ] Content-addressable hashing
- [ ] Tests: put, get, hash stability

### Phase 2: File-Based Implementation (3 hours)

- [ ] Directory structure (`~/.trust-construction/values/`)
- [ ] JSON serialization
- [ ] Hash-based filenames
- [ ] Deduplication automatic (same content = same hash)
- [ ] Tests: persistence, dedup

### Phase 3: Index Trait (2 hours)

- [ ] Create `src/index.rs`
- [ ] Define `Index` trait
- [ ] Common query patterns
- [ ] Rebuild interface
- [ ] Tests: query, rebuild

### Phase 4: SQLite as Index (3 hours)

- [ ] Refactor `db.rs` → index-only schema
- [ ] Remove UPDATE/DELETE (only INSERT for indexing)
- [ ] Implement `rebuild()` from value store
- [ ] Tests: index queries, rebuild

### Phase 5: Integration (2 hours)

- [ ] App uses ValueStore + Index
- [ ] Queries use index → fetch from value store
- [ ] Can drop index and rebuild
- [ ] Tests: end-to-end

---

## 🧪 Criterios de Éxito

```rust
#[test]
fn test_value_store_is_immutable() {
    let store = FileValueStore::new("./test-values");

    let tx = Transaction::new(...);
    let hash1 = store.put(&tx);

    // Try to modify (shouldn't be possible)
    // No put_mut(), no delete(), no update()

    // ✅ Same content = same hash (dedup automatic)
    let hash2 = store.put(&tx);
    assert_eq!(hash1, hash2);

    // ✅ Can retrieve by hash
    let retrieved = store.get(&hash1).unwrap();
    assert_eq!(retrieved, tx);
}

#[test]
fn test_index_is_rebuild able() {
    let store = FileValueStore::new("./test-values");
    let mut index = SqliteIndex::new(":memory:");

    // Import 100 transactions
    for tx in test_transactions(100) {
        let hash = store.put(&tx);
        index.index_transaction(&hash, &tx);
    }

    // Query works
    let results = index.by_merchant("Starbucks");
    assert_eq!(results.len(), 10);

    // ✅ Drop index completely
    drop(index);
    let mut index = SqliteIndex::new(":memory:");

    // ✅ Rebuild from value store
    index.rebuild(&store);

    // ✅ Same results after rebuild
    let results = index.by_merchant("Starbucks");
    assert_eq!(results.len(), 10);
}

#[test]
fn test_operational_flexibility() {
    let store = FileValueStore::new("./test-values");

    // Can switch index implementations
    let index1 = SqliteIndex::new(":memory:");
    let index2 = InMemoryIndex::new();
    let index3 = ElasticsearchIndex::new();

    // All rebuild from same value store
    index1.rebuild(&store);
    index2.rebuild(&store);
    index3.rebuild(&store);

    // ✅ Values are forever, indexes are ephemeral
}
```

---

## 🎁 Benefits

1. **Operational Flexibility** - Change index tech without migration
2. **Zero Data Loss** - Values never deleted, only indexes
3. **Deduplication Automatic** - Same content = same hash
4. **Backup Simple** - Just backup value store
5. **Disaster Recovery** - Rebuild indexes from values
6. **Testing** - Test with different index implementations

---

## 📊 Metrics

- [ ] 0 UPDATE statements in entire codebase
- [ ] 0 DELETE statements in entire codebase
- [ ] Index can rebuild from scratch in <5 seconds
- [ ] Tests: 25+ value store tests

---

**Previous:** Badge 27 - Pure Import
**Next:** Badge 29 - Schema Refinement

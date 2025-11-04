# 🏗️ Fact Model Architecture - Rich Hickey Philosophy

**Date:** 2025-11-03
**Status:** 🚧 IN PROGRESS
**Based on:** Rich Hickey's "Deconstructing the Database"

---

## 🎯 Core Philosophy

> "Information systems should be fundamentally about facts. Maintaining facts, manipulating facts, and presenting facts to users."
> 
> "Traditional schemas confuse three orthogonal concerns: what facts mean, their shape, and what's required in specific contexts."

---

## 🏛️ The 3-Layer Architecture

### Layer 1: Semantic Layer (Independent Attributes)

**What it is:**
- Catalog of ALL possible attributes in the system
- Each attribute is a **first-class citizen**
- Attributes exist independently of entities
- Attributes are **related to** entities, not **owned by** them

**Example:**
```rust
// Attribute Registry
{
  "date": {
    "id": "attr:date",
    "name": "date",
    "type": "DateTime",
    "description": "When the transaction occurred",
    "validation": "ISO-8601 format",
    "provenance": "Extracted from source document"
  },
  "amount": {
    "id": "attr:amount",
    "name": "amount",
    "type": "Decimal",
    "description": "Transaction amount in USD",
    "validation": "Non-zero number",
    "provenance": "Extracted and normalized"
  },
  "merchant": {
    "id": "attr:merchant",
    "name": "merchant",
    "type": "String",
    "description": "Merchant name",
    "validation": "Non-empty string",
    "provenance": "Extracted via pattern matching"
  }
}
```

**Key principle:** Attributes are **reusable** across different entities.

---

### Layer 2: Shape Layer (Schemas)

**What it is:**
- Defines which attributes can appear together
- Schemas **reference** attributes, they don't **define** them
- Multiple schemas can reference the same attribute
- Schemas describe **allowable shapes**, not ownership

**Example (CUE):**
```cue
// Transaction schema - references attributes
Transaction: {
  // Required core attributes
  date:        attr.date
  amount:      attr.amount
  description: attr.description
  
  // Optional attributes
  merchant?:   attr.merchant
  category?:   attr.category
  
  // Provenance attributes (always required)
  source_file:  attr.source_file
  source_line:  attr.source_line
  extracted_at: attr.extracted_at
}

// RawTransaction schema - different shape, same attributes
RawTransaction: {
  date:        attr.date        // Same attribute!
  amount:      attr.amount      // Same attribute!
  raw_line:    attr.raw_line
}
```

**Key principle:** Schemas aggregate references to attributes to describe a shape.

---

### Layer 3: Context Layer (Selections)

**What it is:**
- Defines what's **required** in specific contexts
- Same entity can have different requirements in different contexts
- Context determines which attributes are mandatory

**Example:**
```cue
// Context: UI Display
context_ui: Transaction & {
  // For UI, these are REQUIRED
  date!:     _
  merchant!: _
  amount!:   _
  
  // These are optional
  category?: _
}

// Context: Audit Trail
context_audit: Transaction & {
  // For audit, provenance is REQUIRED
  source_file!:  _
  source_line!:  _
  extracted_at!: _
  parser_version!: _
  
  // Core data is optional (might be invalid)
  date?:     _
  merchant?: _
}

// Context: Financial Report
context_report: Transaction & {
  // For reports, these are REQUIRED
  date!:     _
  amount!:   _
  category!: _
  
  // Merchant is optional
  merchant?: _
}
```

**Key principle:** Context determines requirements, not the schema.

---

## 📊 Comparison: Old vs New

### ❌ OLD (Object-based)

```rust
// Rigid struct - mixing all concerns
pub struct Transaction {
    // What does this mean? → Semantic
    pub date: String,
    pub amount: f64,
    pub merchant: String,
    
    // What shape? → Structure  
    // (hardcoded in struct definition)
    
    // What's required? → Context
    // (everything is always required - no flexibility)
}

// Problems:
// 1. Can't add fields without modifying struct
// 2. Can't have different requirements in different contexts
// 3. Can't reuse attributes across entities
// 4. Schema owns the attributes
```

---

### ✅ NEW (Fact-based)

```rust
// 1. Semantic Layer - Attribute Registry
pub struct AttributeDefinition {
    pub id: String,           // "attr:date"
    pub name: String,         // "date"
    pub type_: AttributeType, // DateTime
    pub description: String,
    pub validation: String,
}

pub struct AttributeRegistry {
    attributes: HashMap<String, AttributeDefinition>,
}

// 2. Shape Layer - Schema (references attributes)
pub struct Schema {
    pub name: String,                          // "Transaction"
    pub attributes: Vec<AttributeRef>,         // References to registry
    pub optional_attributes: Vec<AttributeRef>,
}

// 3. Context Layer - Selection (requirements per context)
pub struct ContextSelection {
    pub context: String,                    // "ui", "audit", "report"
    pub required_attributes: Vec<String>,   // ["date", "amount"]
    pub optional_attributes: Vec<String>,   // ["merchant"]
}

// 4. Fact Storage - Entity-Attribute-Value
pub struct Fact {
    pub entity_id: String,      // "tx:12345"
    pub attribute_id: String,   // "attr:date"
    pub value: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub provenance: Provenance,
}
```

**Benefits:**
1. ✅ Attributes are independent, reusable
2. ✅ Schemas reference attributes, don't own them
3. ✅ Same entity can have different requirements in different contexts
4. ✅ Can add new attributes without changing code
5. ✅ Fully auditable (every fact has provenance)

---

## 🔄 Migration Strategy

### Phase 1: Parallel Systems (Current)

```
┌─────────────────────────────────────┐
│ OLD: Transaction struct (existing)  │
│ - Keep for backward compatibility   │
│ - Read-only                          │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│ NEW: Fact-based storage (parallel)  │
│ - Write all new data here            │
│ - Attribute Registry                 │
│ - CUE Schemas                        │
│ - Context Selections                 │
└─────────────────────────────────────┘
```

---

### Phase 2: Gradual Migration

1. **Read from OLD, write to NEW**
   - Keep existing Transaction struct
   - All writes go to fact storage
   - Queries read from both

2. **Migrate one entity at a time**
   - Start with new transactions
   - Slowly migrate old data
   - No downtime

3. **Deprecate OLD**
   - Once all data migrated
   - Remove old struct
   - 100% fact-based

---

## 🎯 Implementation Plan

### Step 1: Attribute Registry (Semantic Layer)

**File:** `src/attributes.rs`

```rust
pub struct AttributeDefinition {
    pub id: String,
    pub name: String,
    pub type_: AttributeType,
    pub description: String,
    pub validation_rules: Vec<ValidationRule>,
    pub provenance_info: String,
}

pub enum AttributeType {
    String,
    Number,
    DateTime,
    Boolean,
    Json,
}

pub struct AttributeRegistry {
    attributes: HashMap<String, AttributeDefinition>,
}

impl AttributeRegistry {
    pub fn new() -> Self {
        let mut registry = AttributeRegistry {
            attributes: HashMap::new(),
        };
        
        // Register all known attributes
        registry.register_core_attributes();
        registry
    }
    
    fn register_core_attributes(&mut self) {
        self.register(AttributeDefinition {
            id: "attr:date".to_string(),
            name: "date".to_string(),
            type_: AttributeType::DateTime,
            description: "Transaction date".to_string(),
            validation_rules: vec![ValidationRule::Required],
            provenance_info: "Extracted from source document".to_string(),
        });
        
        // ... register all other attributes
    }
    
    pub fn register(&mut self, attr: AttributeDefinition) {
        self.attributes.insert(attr.id.clone(), attr);
    }
    
    pub fn get(&self, id: &str) -> Option<&AttributeDefinition> {
        self.attributes.get(id)
    }
}
```

---

### Step 2: CUE Schemas (Shape Layer)

**File:** `schemas/transaction.cue`

```cue
package transaction

// Attribute imports (from registry)
#Attribute: {
  id:          string
  name:        string
  type:        "String" | "Number" | "DateTime" | "Boolean" | "Json"
  description: string
}

// Core attributes
#date: #Attribute & {
  id:   "attr:date"
  name: "date"
  type: "DateTime"
}

#amount: #Attribute & {
  id:   "attr:amount"
  name: "amount"
  type: "Number"
}

// Transaction schema - references attributes
#Transaction: {
  // Required attributes
  date:        #date
  amount:      #amount
  description: string
  
  // Optional attributes
  merchant?:   string
  category?:   string
  
  // Provenance (always required)
  source_file:  string
  source_line:  int
  extracted_at: string
  
  // Extensible metadata
  metadata?: {[string]: _}
}

// Validation
transaction: #Transaction
```

---

### Step 3: Context Selections (Context Layer)

**File:** `schemas/contexts.cue`

```cue
package contexts

import "transaction"

// UI Context - what's needed for display
#UIContext: transaction.#Transaction & {
  // MUST have these for UI
  date!:     _
  merchant!: _
  amount!:   _
  
  // Optional in UI
  category?: _
}

// Audit Context - what's needed for compliance
#AuditContext: transaction.#Transaction & {
  // MUST have provenance
  source_file!:     _
  source_line!:     _
  extracted_at!:    _
  parser_version!:  _
  
  // Data can be incomplete
  date?:     _
  merchant?: _
}

// Report Context - what's needed for financial reports
#ReportContext: transaction.#Transaction & {
  // MUST have for reports
  date!:     _
  amount!:   _
  category!: _
  
  // Optional
  merchant?: _
}
```

---

### Step 4: Fact Storage

**File:** `src/facts.rs`

```rust
pub struct Fact {
    pub entity_id: String,      // "tx:12345"
    pub attribute_id: String,   // "attr:date"
    pub value: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub provenance: Provenance,
}

pub struct Provenance {
    pub source_file: String,
    pub source_line: usize,
    pub extracted_at: DateTime<Utc>,
    pub parser_version: String,
    pub transformation_log: Vec<String>,
}

pub struct FactStore {
    conn: rusqlite::Connection,
}

impl FactStore {
    pub fn insert_fact(&self, fact: &Fact) -> Result<()> {
        // Insert into facts table
        self.conn.execute(
            "INSERT INTO facts (entity_id, attribute_id, value, timestamp, provenance)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &fact.entity_id,
                &fact.attribute_id,
                serde_json::to_string(&fact.value)?,
                &fact.timestamp.to_rfc3339(),
                serde_json::to_string(&fact.provenance)?,
            ],
        )?;
        Ok(())
    }
    
    pub fn get_entity(&self, entity_id: &str) -> Result<HashMap<String, Fact>> {
        // Reconstruct entity from facts
        let mut stmt = self.conn.prepare(
            "SELECT attribute_id, value, timestamp, provenance
             FROM facts
             WHERE entity_id = ?1
             ORDER BY timestamp DESC"
        )?;
        
        let facts = stmt.query_map(params![entity_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                Fact {
                    entity_id: entity_id.to_string(),
                    attribute_id: row.get(0)?,
                    value: serde_json::from_str(&row.get::<_, String>(1)?)?,
                    timestamp: row.get(2)?,
                    provenance: serde_json::from_str(&row.get::<_, String>(3)?)?,
                }
            ))
        })?;
        
        facts.collect()
    }
}
```

---

## 📈 Benefits of Fact Model

### 1. Decomplecting

**Before:** Schema confuses 3 concerns
```
Transaction struct = {
  Semantic (what does "date" mean?)
  + Shape (which fields go together?)
  + Context (what's required when?)
}
```

**After:** 3 independent layers
```
Semantic Layer → Attribute Registry (what attributes mean)
Shape Layer    → CUE Schemas (which attributes together)
Context Layer  → Selections (what's required when)
```

---

### 2. Extensibility

**Before:**
```rust
// Want to add confidence_score?
// → Must modify struct ❌
// → Must update SQL schema ❌
// → Must migrate data ❌
```

**After:**
```rust
// Add confidence_score
registry.register(AttributeDefinition {
    id: "attr:confidence_score",
    name: "confidence_score",
    type_: AttributeType::Number,
    // ...
});

// Use it immediately in any schema ✅
// No migration needed ✅
```

---

### 3. Context Flexibility

**Before:**
```rust
// Same requirements everywhere
struct Transaction {
    pub date: String,        // Always required
    pub merchant: String,    // Always required
}
```

**After:**
```cue
// Different requirements per context
#UIContext: {
    merchant!: _  // Required for UI
}

#AuditContext: {
    merchant?: _  // Optional for audit (might be unknown)
}
```

---

### 4. Auditability

**Every fact has provenance:**
```rust
Fact {
    entity_id: "tx:12345",
    attribute_id: "attr:merchant",
    value: "STARBUCKS",
    provenance: {
        source_file: "bofa_march.csv",
        extracted_at: "2024-03-20T10:30:00Z",
        parser_version: "bofa_parser_v1.0",
        transformation_log: ["extracted", "normalized", "validated"]
    }
}
```

**Questions we can answer:**
- "What did we know about this transaction on March 20?"
- "Which parser version extracted this merchant?"
- "Has this value ever changed?"

---

## 🎯 Next Steps

1. ✅ Create attribute registry
2. ✅ Define CUE schemas
3. ✅ Implement context selections
4. ✅ Create fact storage
5. ✅ Migrate Transaction to use facts
6. ✅ Tests for all 3 layers

**Status:** Ready to implement!

---

**Last Updated:** 2025-11-03
**Next:** Implement attribute registry in Rust

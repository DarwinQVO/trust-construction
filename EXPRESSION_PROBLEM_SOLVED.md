# 🎯 Expression Problem - SOLVED

**Fecha:** 2025-11-03
**Badge:** 6 (Refactored)
**Principio:** Polimorfismo à la Carte

---

## 🔥 El Problema

**Expression Problem** (Philip Wadler, 1998):

> "Can you extend a system with new operations AND new types without modifying existing code?"

**En términos simples:**

```
Sistema = DATOS + FUNCIONES

Extensibilidad en 2 dimensiones:
1. Agregar nuevos DATOS (tipos, bancos)
2. Agregar nuevas FUNCIONES (operaciones)

Sin modificar código existente
Sin cerrar la puerta a futuras evoluciones
```

---

## ❌ Solución Monolítica (Badge 6 v1.0)

### Diseño Original

```rust
pub trait BankParser {
    fn parse(&self, ...) -> Result<Vec<RawTransaction>>;
    fn source_type(&self) -> SourceType;
    fn can_parse(&self, ...) -> bool;
    fn extract_merchant(&self, ...) -> Option<String>;
    fn classify_type(&self, ...) -> String;
    fn version(&self) -> &str;
}
```

### Problema 1: Agregar TIPOS (bancos) ✅

```rust
// ✅ FÁCIL: Implementar trait para Chase
impl BankParser for ChaseParser {
    fn parse(&self, ...) { ... }
    fn source_type(&self) { ... }
    fn can_parse(&self, ...) { ... }
    fn extract_merchant(&self, ...) { ... }
    fn classify_type(&self, ...) { ... }
}
```

**Resultado:** ✅ No toca código existente

### Problema 2: Agregar FUNCIONES ❌

```rust
// ❌ DIFÍCIL: Agregar validate_amount()
pub trait BankParser {
    // ... métodos existentes
    fn validate_amount(&self, ...) -> Result<f64>;  // ← NUEVO
}

// ❌ ROMPE TODOS LOS PARSERS EXISTENTES
impl BankParser for BofAParser {
    // Error: missing method `validate_amount`
}
```

**Resultado:** ❌ Modifica código existente (todos los impl)

---

## ✅ Solución Composable (Badge 6 v2.0)

### Diseño Refactorizado

```rust
// 1. Core trait (minimal, required)
pub trait BankParser: Send + Sync {
    fn parse(&self, file_path: &Path) -> Result<Vec<RawTransaction>>;
    fn source_type(&self) -> SourceType;
    fn version(&self) -> &str { "1.0.0" }
}

// 2. Optional traits (composable)
pub trait FileValidator {
    fn can_parse(&self, file_path: &Path) -> bool;
}

pub trait MerchantExtractor {
    fn extract_merchant(&self, description: &str) -> Option<String>;
}

pub trait TypeClassifier {
    fn classify_type(&self, description: &str, amount: f64) -> String;
}

// 3. Future extensions (examples)
pub trait AmountValidator {
    fn validate_amount(&self, amount: &str) -> Result<f64>;
}

pub trait DateNormalizer {
    fn normalize_date(&self, date: &str) -> Result<String>;
}
```

### Solución 1: Agregar TIPOS (bancos) ✅

```rust
// ✅ FÁCIL: Implementar solo BankParser (core)
impl BankParser for ChaseParser {
    fn parse(&self, ...) { ... }
    fn source_type(&self) { SourceType::Chase }
}

// ✅ OPCIONAL: Agregar capabilities
impl MerchantExtractor for ChaseParser {
    fn extract_merchant(&self, ...) { ... }
}

// ✅ NO necesita implementar TODAS las capabilities
```

**Resultado:** ✅ No toca código existente

### Solución 2: Agregar FUNCIONES ✅

```rust
// ✅ FÁCIL: Crear nuevo trait
pub trait AmountValidator {
    fn validate_amount(&self, amount: &str) -> Result<f64>;
}

// ✅ Implementar en parsers que lo necesiten
impl AmountValidator for BofAParser {
    fn validate_amount(&self, amount: &str) -> Result<f64> {
        amount.parse::<f64>()
            .map_err(|e| anyhow!("Invalid amount: {}", e))
    }
}

// ✅ Parsers existentes NO necesitan cambiar
// BofAParser sigue funcionando sin AmountValidator
```

**Resultado:** ✅ No modifica código existente

---

## 📊 Comparación

| Dimensión              | Monolítico | Composable |
|------------------------|------------|------------|
| Agregar TIPO (banco)   | ✅ Fácil    | ✅ Fácil    |
| Agregar FUNCIÓN (trait)| ❌ Rompe    | ✅ Fácil    |
| Expression Problem     | ❌ NO       | ✅ SÍ       |

---

## 🎨 Patrón: Polimorfismo à la Carte

### Principio

**"Pick and choose capabilities, don't force a monolithic interface"**

### Ejemplo Real

```rust
// Parser mínimo (solo parsing)
struct SimpleParser;

impl BankParser for SimpleParser {
    fn parse(&self, ...) { ... }
    fn source_type(&self) { ... }
}
// ✅ LISTO! No necesita merchant extraction ni classification


// Parser completo (todas las capabilities)
struct AdvancedParser;

impl BankParser for AdvancedParser { ... }
impl MerchantExtractor for AdvancedParser { ... }
impl TypeClassifier for AdvancedParser { ... }
impl AmountValidator for AdvancedParser { ... }
// ✅ Todas las capabilities disponibles
```

### Ventajas

1. **Flexibilidad** - Cada parser decide qué implementar
2. **Extensibilidad** - Agregar capabilities sin romper nada
3. **Minimalismo** - Core trait es mínimo (solo 3 métodos)
4. **Documentación** - Cada trait documenta una capability específica

---

## 🔧 Implementación

### Core Trait (Obligatorio)

```rust
pub trait BankParser: Send + Sync {
    /// Parse file → transactions (REQUIRED)
    fn parse(&self, file_path: &Path) -> Result<Vec<RawTransaction>>;

    /// Identify source (REQUIRED)
    fn source_type(&self) -> SourceType;

    /// Parser version (OPTIONAL, default provided)
    fn version(&self) -> &str {
        "1.0.0"
    }
}
```

**Características:**
- Solo 2 métodos obligatorios
- 1 método opcional con default
- Send + Sync para threading

### Extension Traits (Opcionales)

```rust
/// Capability: File validation
pub trait FileValidator {
    fn can_parse(&self, file_path: &Path) -> bool;
}

/// Capability: Merchant extraction
pub trait MerchantExtractor {
    fn extract_merchant(&self, description: &str) -> Option<String>;
}

/// Capability: Type classification
pub trait TypeClassifier {
    fn classify_type(&self, description: &str, amount: f64) -> String;
}
```

**Características:**
- 1 método por trait (Single Responsibility)
- Independientes entre sí
- Parser decide cuáles implementar

### Future Extensions (Ejemplos)

```rust
/// Capability: Amount validation
pub trait AmountValidator {
    fn validate_amount(&self, amount: &str) -> Result<f64>;
}

/// Capability: Date normalization
pub trait DateNormalizer {
    fn normalize_date(&self, date: &str) -> Result<String>;
}

/// Capability: Category inference (ML)
pub trait CategoryInferrer {
    fn infer_category(&self, merchant: &str, amount: f64) -> Option<String>;
}
```

**Características:**
- Definidos pero no implementados aún
- Documentan futuras extensiones
- No afectan parsers existentes

---

## 📝 Casos de Uso

### Caso 1: Parser Simple (CSV básico)

```rust
struct BasicCSVParser;

// Solo implementa lo mínimo
impl BankParser for BasicCSVParser {
    fn parse(&self, path: &Path) -> Result<Vec<RawTransaction>> {
        // Lee CSV, extrae campos básicos
        Ok(transactions)
    }

    fn source_type(&self) -> SourceType {
        SourceType::BankOfAmerica
    }
}

// ✅ Funciona sin merchant extraction ni classification
```

### Caso 2: Parser Inteligente (ML-powered)

```rust
struct SmartParser {
    ml_model: CategoryModel,
}

impl BankParser for SmartParser {
    fn parse(&self, path: &Path) -> Result<Vec<RawTransaction>> { ... }
    fn source_type(&self) -> SourceType { ... }
}

impl MerchantExtractor for SmartParser {
    fn extract_merchant(&self, desc: &str) -> Option<String> {
        // Regex avanzado + NLP
        Some(merchant)
    }
}

impl TypeClassifier for SmartParser {
    fn classify_type(&self, desc: &str, amount: f64) -> String {
        // Usa ML model
        self.ml_model.predict(desc, amount)
    }
}

impl CategoryInferrer for SmartParser {
    fn infer_category(&self, merchant: &str, amount: f64) -> Option<String> {
        // Inferencia con ML
        self.ml_model.infer_category(merchant, amount)
    }
}

// ✅ Todas las capabilities disponibles
```

### Caso 3: Agregar Nueva Capability (Futuro)

```rust
// Nueva capability: Currency conversion
pub trait CurrencyConverter {
    fn convert_to_usd(&self, amount: f64, currency: &str) -> Result<f64>;
}

// Implementar en Wise parser (maneja múltiples currencies)
impl CurrencyConverter for WiseParser {
    fn convert_to_usd(&self, amount: f64, currency: &str) -> Result<f64> {
        // API call o lookup table
        Ok(converted_amount)
    }
}

// ✅ Otros parsers NO necesitan implementar esto
// ✅ No modifica código existente
```

---

## 🎯 Expression Problem Coverage

### Dimensión 1: Agregar TIPOS ✅

**Ejemplo: Agregar Chase Bank**

```rust
// 1. Crear struct
struct ChaseParser;

// 2. Implementar core trait
impl BankParser for ChaseParser {
    fn parse(&self, path: &Path) -> Result<Vec<RawTransaction>> {
        // Parse Chase CSV format
        Ok(transactions)
    }

    fn source_type(&self) -> SourceType {
        SourceType::Chase  // Agregar variant al enum
    }
}

// 3. (Opcional) Implementar capabilities
impl MerchantExtractor for ChaseParser {
    fn extract_merchant(&self, desc: &str) -> Option<String> {
        // Chase-specific extraction
        Some(merchant)
    }
}

// ✅ NO toca: BofAParser, AppleCardParser, etc.
// ✅ NO modifica: Ningún código existente
```

### Dimensión 2: Agregar FUNCIONES ✅

**Ejemplo: Agregar validate_amount()**

```rust
// 1. Crear nuevo trait
pub trait AmountValidator {
    fn validate_amount(&self, amount: &str) -> Result<f64>;
}

// 2. Implementar en parsers que lo necesitan
impl AmountValidator for BofAParser {
    fn validate_amount(&self, amount: &str) -> Result<f64> {
        amount.parse::<f64>()
            .map_err(|e| anyhow!("Invalid amount: {}", e))
    }
}

impl AmountValidator for WiseParser {
    fn validate_amount(&self, amount: &str) -> Result<f64> {
        // Wise-specific validation (handles decimals differently)
        validate_wise_amount(amount)
    }
}

// ✅ AppleCardParser NO necesita implementar AmountValidator
// ✅ AppleCardParser sigue funcionando sin cambios
// ✅ NO modifica código existente
```

---

## 🏆 Beneficios

### 1. Extensibilidad Total

```rust
// Agregar banco = Implementar BankParser
// Agregar función = Crear nuevo trait
// NO tocar código existente
```

### 2. Minimalismo

```rust
// Core trait = Solo 2 métodos obligatorios
// Todo lo demás = Opcional
```

### 3. Type Safety

```rust
// Compiler verifica implementaciones
// Si falta método obligatorio → Error en compile time
// Si capability no implementada → No disponible (seguro)
```

### 4. Documentación Explícita

```rust
// Cada trait = 1 capability clara
// MerchantExtractor = Extrae merchants
// TypeClassifier = Clasifica tipos
// Autodocumentado
```

### 5. Testing Granular

```rust
#[test]
fn test_merchant_extraction() {
    let parser = BofAParser::new();
    assert_eq!(parser.extract_merchant("VISA STARBUCKS"), Some("STARBUCKS"));
}

#[test]
fn test_type_classification() {
    let parser = BofAParser::new();
    assert_eq!(parser.classify_type("PURCHASE", -45.99), "GASTO");
}

// Cada capability = Tests independientes
```

---

## 📚 Teoría: Expression Problem

### Historia

**Definición original (Philip Wadler, 1998):**

> "The goal is to define a datatype by cases, where one can add new cases to the datatype and new functions over the datatype, without recompiling existing code, and while retaining static type safety."

### Soluciones en Diferentes Lenguajes

| Lenguaje       | Solución                              | Limitación                  |
|----------------|---------------------------------------|-----------------------------|
| OOP (Java)     | Subclasses                            | ❌ Agregar funciones difícil |
| FP (Haskell)   | Pattern matching                      | ❌ Agregar tipos difícil     |
| Rust (Traits)  | Composable traits                     | ✅ Ambas dimensiones         |

### Por Qué Rust Lo Resuelve

**Características clave:**

1. **Traits separables** - No monolíticos
2. **Impl independientes** - No modifica structs
3. **Type safety** - Compiler verifica todo
4. **Default methods** - Retrocompatibilidad

---

## 🎓 Lecciones Aprendidas

### 1. Traits Pequeños > Traits Grandes

**Malo:**
```rust
trait Parser {
    fn method1(...);
    fn method2(...);
    fn method3(...);
    // ... 10 métodos
}
```

**Bueno:**
```rust
trait CoreParser { fn parse(...); }
trait Extractor { fn extract(...); }
trait Classifier { fn classify(...); }
```

### 2. Obligatorio vs Opcional

**Malo:** Todo obligatorio
```rust
trait Parser {
    fn parse(...);           // Todos deben implementar
    fn extract_merchant(...); // Todos deben implementar
    fn classify(...);         // Todos deben implementar
}
```

**Bueno:** Core obligatorio, resto opcional
```rust
trait Parser { fn parse(...); }  // Obligatorio
trait Extractor { ... }          // Opcional
trait Classifier { ... }         // Opcional
```

### 3. Documentar Extensiones Futuras

```rust
// ✅ Define traits para futuro
pub trait AmountValidator { ... }  // Aún no implementado
pub trait DateNormalizer { ... }   // Aún no implementado

// Beneficios:
// 1. Documenta intenciones
// 2. Guía implementaciones futuras
// 3. No afecta código actual
```

---

## ✅ Verificación

### Checklist: Expression Problem Solved

- [x] ¿Puedo agregar banco sin tocar código? **SÍ** ✅
- [x] ¿Puedo agregar función sin tocar parsers? **SÍ** ✅
- [x] ¿Compiler verifica correctitud? **SÍ** ✅
- [x] ¿Tests pasan sin cambios? **SÍ** ✅
- [x] ¿Código existente funciona? **SÍ** ✅

**Expression Problem: SOLVED** ✅

---

## 🚀 Próximos Pasos

### Badge 7: BofA Parser

```rust
impl BankParser for BofAParser {
    fn parse(&self, path: &Path) -> Result<Vec<RawTransaction>> {
        // IMPLEMENTAR: Leer CSV BofA
    }
}

impl MerchantExtractor for BofAParser {
    fn extract_merchant(&self, desc: &str) -> Option<String> {
        // IMPLEMENTAR: "DEBIT PURCHASE -VISA STARBUCKS" → "STARBUCKS"
    }
}

impl TypeClassifier for BofAParser {
    fn classify_type(&self, desc: &str, amount: f64) -> String {
        // IMPLEMENTAR: Detectar GASTO/INGRESO/TRASPASO
    }
}
```

### Futuro: Extensiones

```rust
// Badge 15+: Agregar ML classification
impl CategoryInferrer for SmartParser { ... }

// Badge 18+: Agregar currency conversion
impl CurrencyConverter for WiseParser { ... }

// ✅ Sin modificar Badge 6-14 code
```

---

✅ **Expression Problem SOLVED** - Sistema extensible en ambas dimensiones! 🎉

*"Polimorfismo à la Carte: Pick capabilities, don't force monoliths"*

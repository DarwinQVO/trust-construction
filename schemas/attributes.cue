// 🏛️ Semantic Layer - Attribute Definitions (CUE)
// References to attributes from the Rust AttributeRegistry

package transaction

// Attribute type definition
#Attribute: {
    id:          string
    name:        string
    type:        "String" | "Number" | "DateTime" | "Boolean" | "Json"
    description: string
    examples:    [...string]
}

// ============================================================================
// TEMPORAL ATTRIBUTES
// ============================================================================

#date: #Attribute & {
    id:   "attr:date"
    name: "date"
    type: "DateTime"
    description: "Transaction date - when the transaction occurred"
    examples: ["01/15/2024", "2024-01-15"]
}

#extracted_at: #Attribute & {
    id:   "attr:extracted_at"
    name: "extracted_at"
    type: "DateTime"
    description: "When this data was extracted from source"
    examples: ["2024-01-15T10:30:00Z"]
}

// ============================================================================
// MONETARY ATTRIBUTES
// ============================================================================

#amount: #Attribute & {
    id:   "attr:amount"
    name: "amount"
    type: "Number"
    description: "Transaction amount in USD"
    examples: ["45.99", "-120.50"]
}

#amount_original: #Attribute & {
    id:   "attr:amount_original"
    name: "amount_original"
    type: "String"
    description: "Original amount string from source (before parsing)"
    examples: ["-$45.99", "120.50 USD"]
}

#currency: #Attribute & {
    id:   "attr:currency"
    name: "currency"
    type: "String"
    description: "Currency code"
    examples: ["USD", "EUR", "MXN"]
}

// ============================================================================
// DESCRIPTIVE ATTRIBUTES
// ============================================================================

#description: #Attribute & {
    id:   "attr:description"
    name: "description"
    type: "String"
    description: "Transaction description from source"
    examples: ["STARBUCKS STORE #12345"]
}

#merchant: #Attribute & {
    id:   "attr:merchant"
    name: "merchant"
    type: "String"
    description: "Extracted merchant name"
    examples: ["STARBUCKS", "AMAZON"]
}

// ============================================================================
// CLASSIFICATION ATTRIBUTES
// ============================================================================

#transaction_type: #Attribute & {
    id:   "attr:transaction_type"
    name: "transaction_type"
    type: "String"
    description: "Transaction type classification"
    examples: ["GASTO", "INGRESO", "PAGO_TARJETA", "TRASPASO"]
}

#category: #Attribute & {
    id:   "attr:category"
    name: "category"
    type: "String"
    description: "Transaction category"
    examples: ["Restaurants", "Transportation"]
}

// ============================================================================
// PROVENANCE ATTRIBUTES
// ============================================================================

#source_file: #Attribute & {
    id:   "attr:source_file"
    name: "source_file"
    type: "String"
    description: "Original source file name"
    examples: ["bofa_march_2024.csv"]
}

#source_line: #Attribute & {
    id:   "attr:source_line"
    name: "source_line"
    type: "Number"
    description: "Line number in source file"
    examples: ["23"]
}

#parser_version: #Attribute & {
    id:   "attr:parser_version"
    name: "parser_version"
    type: "String"
    description: "Version of parser that extracted this data"
    examples: ["bofa_parser_v1.0"]
}

// ============================================================================
// CONFIDENCE & VERIFICATION ATTRIBUTES
// ============================================================================

#confidence_score: #Attribute & {
    id:   "attr:confidence_score"
    name: "confidence_score"
    type: "Number"
    description: "Confidence score for classification (0.0-1.0)"
    examples: ["0.95"]
}

#verified: #Attribute & {
    id:   "attr:verified"
    name: "verified"
    type: "Boolean"
    description: "Whether transaction has been manually verified"
    examples: ["true", "false"]
}

#verified_by: #Attribute & {
    id:   "attr:verified_by"
    name: "verified_by"
    type: "String"
    description: "Who verified this transaction"
    examples: ["user_123"]
}

#verified_at: #Attribute & {
    id:   "attr:verified_at"
    name: "verified_at"
    type: "DateTime"
    description: "When transaction was verified"
    examples: ["2024-03-20T15:30:00Z"]
}

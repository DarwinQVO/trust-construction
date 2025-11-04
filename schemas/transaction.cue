// 📐 Shape Layer - Transaction Schema
// Defines which attributes can appear together
// Schemas REFERENCE attributes, they don't OWN them

package transaction

// Transaction schema - references attributes from attribute registry
#Transaction: {
    // ================================================================
    // REQUIRED CORE ATTRIBUTES (always present)
    // ================================================================
    
    // Temporal
    date:        string  // References #date attribute
    
    // Monetary
    amount:      number  // References #amount attribute
    description: string  // References #description attribute
    
    // Provenance (ALWAYS required for trust construction)
    source_file:  string  // References #source_file
    source_line:  int & >0  // References #source_line (must be positive)
    extracted_at: string  // References #extracted_at
    
    // ================================================================
    // OPTIONAL ATTRIBUTES (may or may not be present)
    // ================================================================
    
    // Monetary (optional)
    amount_original?: string   // References #amount_original
    currency?:        string   // References #currency
    
    // Descriptive (optional)
    merchant?: string          // References #merchant
    
    // Classification (optional)
    transaction_type?: string  // References #transaction_type
    category?:         string  // References #category
    
    // Provenance (optional)
    parser_version?: string    // References #parser_version
    
    // Confidence (optional)
    confidence_score?: number & >=0 & <=1  // References #confidence_score
    
    // Verification (optional)
    verified?:    bool    // References #verified
    verified_by?: string  // References #verified_by
    verified_at?: string  // References #verified_at
    
    // ================================================================
    // EXTENSIBLE METADATA (key-value pairs)
    // ================================================================
    
    // Any additional attributes can go here
    metadata?: {[string]: _}  // Fully extensible
}

// RawTransaction schema - different shape, same attributes
#RawTransaction: {
    // Core fields
    date:        string    // Same attribute as Transaction!
    description: string    // Same attribute as Transaction!
    amount:      string    // Same attribute as Transaction!
    
    // Provenance
    source_file: string
    source_line: int & >0
    raw_line:    string    // Raw line from source
    
    // Optional
    merchant?:  string
    category?:  string
    account?:   string
    confidence_score?: number & >=0 & <=1
}

// Example validation - a transaction instance
transaction: #Transaction & {
    date:        "01/15/2024"
    amount:      45.99
    description: "STARBUCKS"
    source_file: "test.csv"
    source_line: 23
    extracted_at: "2024-01-15T10:30:00Z"
    
    // Optional fields
    merchant:        "STARBUCKS"
    category:        "Restaurants"
    transaction_type: "GASTO"
    confidence_score: 0.95
}

// 🎯 Context Layer - Context-Specific Selections
// Defines what's REQUIRED in specific contexts
// Same entity can have different requirements in different contexts

package contexts

import "."  // Import transaction package

// ============================================================================
// CONTEXT: UI DISPLAY
// ============================================================================

// For UI display, we need user-friendly data
#UIContext: transaction.#Transaction & {
    // MUST have these for UI rendering
    date!:     _
    merchant!: _  // Required - UI shows merchant prominently
    amount!:   _
    
    // Should have (but can be empty string)
    transaction_type!: _
    
    // Optional in UI
    category?:         _
    confidence_score?: _
    
    // Provenance is optional for display (user doesn't see it)
    source_file?:  _
    source_line?:  _
}

// ============================================================================
// CONTEXT: AUDIT TRAIL
// ============================================================================

// For audit/compliance, provenance is critical
#AuditContext: transaction.#Transaction & {
    // MUST have full provenance
    source_file!:     _
    source_line!:     _
    extracted_at!:    _
    parser_version!:  _
    
    // Should have transformation info
    metadata!: {
        transformation_log?: [...string]
        ...
    }
    
    // Core data can be incomplete (might have parsing errors)
    date?:     _
    merchant?: _
    amount?:   _
}

// ============================================================================
// CONTEXT: FINANCIAL REPORT
// ============================================================================

// For financial reports, we need complete financial data
#ReportContext: transaction.#Transaction & {
    // MUST have for financial calculations
    date!:     _
    amount!:   _
    category!: _  // Required for categorized reports
    
    // Should have
    transaction_type!: _
    
    // Optional
    merchant?: _
    
    // Provenance not needed for reports
    source_file?:  _
    source_line?:  _
}

// ============================================================================
// CONTEXT: IMPORT/PARSING
// ============================================================================

// For import, we need minimal data + strong provenance
#ImportContext: transaction.#Transaction & {
    // MUST have source info
    source_file!:  _
    source_line!:  _
    extracted_at!: _
    
    // MUST have raw data
    description!: _
    
    // Amount might not be parsed yet
    amount?: _
    
    // Classification might not be done yet
    merchant?:        _
    category?:        _
    transaction_type?: _
}

// ============================================================================
// CONTEXT: VERIFICATION
// ============================================================================

// For manual verification, we need review data
#VerificationContext: transaction.#Transaction & {
    // MUST show user what they're verifying
    date!:        _
    description!: _
    amount!:      _
    
    // MUST show confidence to help user decide
    confidence_score!: _
    
    // After verification, these become required
    verified?:    bool
    verified_by?: string
    verified_at?: string
    
    // Nice to have
    merchant?: _
    category?: _
}

// ============================================================================
// CONTEXT: MACHINE LEARNING TRAINING
// ============================================================================

// For ML training, we need verified, complete data
#MLTrainingContext: transaction.#Transaction & {
    // MUST be verified for training
    verified!:    true  // Only verified transactions
    verified_by!: _
    
    // MUST have complete classification
    description!:      _
    merchant!:         _
    category!:         _
    transaction_type!: _
    
    // Amount for patterns
    amount!: _
}

// ============================================================================
// CONTEXT: DATA QUALITY CHECK
// ============================================================================

// For Great Expectations / data quality
#QualityContext: transaction.#Transaction & {
    // All fields must exist (can be empty, but must be present)
    date!:             _
    amount!:           _
    description!:      _
    transaction_type!: _
    
    // Provenance must be complete
    source_file!:  _
    source_line!:  _
    extracted_at!: _
    
    // Should have quality metrics
    confidence_score?: number & >=0 & <=1
}

// ============================================================================
// EXAMPLE USAGES
// ============================================================================

// Valid for UI context
ui_transaction: #UIContext & {
    date:             "01/15/2024"
    merchant:         "STARBUCKS"
    amount:           45.99
    transaction_type: "GASTO"
    description:      "STARBUCKS STORE #12345"
    source_file:      "test.csv"
    source_line:      23
    extracted_at:     "2024-01-15T10:30:00Z"
}

// Valid for Audit context (has complete provenance)
audit_transaction: #AuditContext & {
    source_file:     "test.csv"
    source_line:     23
    extracted_at:    "2024-01-15T10:30:00Z"
    parser_version:  "bofa_parser_v1.0"
    
    description: "STARBUCKS"
    amount:      45.99
    date:        "01/15/2024"
    
    metadata: {
        transformation_log: ["parsed", "normalized", "classified"]
    }
}

// Valid for Report context (has category)
report_transaction: #ReportContext & {
    date:             "01/15/2024"
    amount:           45.99
    category:         "Restaurants"
    transaction_type: "GASTO"
    description:      "STARBUCKS"
    source_file:      "test.csv"
    source_line:      23
    extracted_at:     "2024-01-15T10:30:00Z"
}

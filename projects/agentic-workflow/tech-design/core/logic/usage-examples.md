---
id: usage-examples
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Usage examples support semantic source understanding during brownfield standardization."
---

# Argus Usage Examples

Practical examples for using Argus P0 features: Refactoring, Semantic Search, and Framework Support.

---

## Table of Contents
<!-- type: doc lang: markdown -->

1. [Refactoring Examples](#refactoring-examples)
2. [Semantic Search Examples](#semantic-search-examples)
3. [Framework Support Examples](#framework-support-examples)
4. [Integration Workflows](#integration-workflows)
5. [Real-World Scenarios](#real-world-scenarios)

---

## Refactoring Examples
<!-- type: doc lang: markdown -->

### Example 1: Rename Variable Across File

```rust
use cclab-lens::types::{RefactoringEngine, RefactorRequest, RefactorKind, Span};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Source code
    let source = r#"
user_name = "Alice"
print(user_name)
result = user_name.upper()
"#;

    // Create refactoring request
    let request = RefactorRequest {
        kind: RefactorKind::Rename {
            new_name: "display_name".to_string(),
        },
        file: PathBuf::from("example.py"),
        span: Span::new(1, 10), // "user_name" position
        options: Default::default(),
    };

    // Execute refactoring
    let mut engine = RefactoringEngine::new();
    let result = engine.execute(&request, source);

    // Check for errors
    if result.has_errors() {
        for diag in result.diagnostics {
            eprintln!("Error: {}", diag.message);
        }
        return Err("Refactoring failed".into());
    }

    // Apply edits
    let modified = apply_edits(source, result.file_edits);
    println!("Modified code:\n{}", modified);

    Ok(())
}

// Helper to apply edits
fn apply_edits(source: &str, file_edits: HashMap<PathBuf, Vec<TextEdit>>) -> String {
    // Implementation from REFACTORING_API.md
    // ...
}
```

**Output**:
```python
display_name = "Alice"
print(display_name)
result = display_name.upper()
```

---

### Example 2: Extract Complex Expression

```rust
fn extract_calculation() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
total = (price * quantity) * (1 + tax_rate) + shipping_cost
"#;

    let request = RefactorRequest {
        kind: RefactorKind::ExtractVariable {
            name: "subtotal_with_tax".to_string(),
        },
        file: PathBuf::from("calc.py"),
        span: Span::new(9, 43), // "(price * quantity) * (1 + tax_rate)"
        options: Default::default(),
    };

    let mut engine = RefactoringEngine::new();
    let result = engine.execute(&request, source);

    if result.has_changes() {
        let modified = apply_edits(source, result.file_edits);
        println!("{}", modified);
    }

    Ok(())
}
```

**Output**:
```python
subtotal_with_tax = (price * quantity) * (1 + tax_rate)
total = subtotal_with_tax + shipping_cost
```

---

### Example 3: Extract Method from Class

```rust
fn extract_validation_logic() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
class User:
    def process(self, data):
        if not data:
            raise ValueError("Empty data")
        if len(data) < 5:
            raise ValueError("Data too short")
        return data.upper()
"#;

    let request = RefactorRequest {
        kind: RefactorKind::ExtractMethod {
            name: "validate_data".to_string(),
        },
        file: PathBuf::from("user.py"),
        span: Span::new(60, 165), // Validation block
        options: Default::default(),
    };

    let mut engine = RefactoringEngine::new();
    let result = engine.execute(&request, source);

    let modified = apply_edits(source, result.file_edits);
    println!("{}", modified);

    Ok(())
}
```

**Output**:
```python
class User:
    def validate_data(self):
        if not data:
            raise ValueError("Empty data")
        if len(data) < 5:
            raise ValueError("Data too short")

    def process(self, data):
        self.validate_data()
        return data.upper()
```

---

### Example 4: Inline Temporary Variable

```rust
fn inline_temp_variable() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
base_price = product.price
discount = base_price * 0.1
final_price = base_price - discount
"#;

    let request = RefactorRequest {
        kind: RefactorKind::Inline,
        file: PathBuf::from("pricing.py"),
        span: Span::new(1, 11), // "base_price"
        options: Default::default(),
    };

    let mut engine = RefactoringEngine::new();
    let result = engine.execute(&request, source);

    let modified = apply_edits(source, result.file_edits);
    println!("{}", modified);

    Ok(())
}
```

**Output**:
```python
discount = product.price * 0.1
final_price = product.price - discount
```

---

### Example 5: Change Function Signature

```rust
fn add_logging_parameter() -> Result<(), Box<dyn std::error::Error>> {
    let source = "def process(data):\n    return data";

    let changes = SignatureChanges {
        new_params: vec![
            ("logger".to_string(),
             Some("Logger".to_string()),
             Some("None".to_string())),
        ],
        param_order: vec![],
        removed_params: vec![],
        new_return_type: None,
    };

    let request = RefactorRequest {
        kind: RefactorKind::ChangeSignature { changes },
        file: PathBuf::from("processor.py"),
        span: Span::new(0, 19),
        options: Default::default(),
    };

    let mut engine = RefactoringEngine::new();
    let result = engine.execute(&request, source);

    let modified = apply_edits(source, result.file_edits);
    println!("{}", modified);

    Ok(())
}
```

**Output**:
```python
def process(data, logger: Logger = None):
    return data
```

---

## Semantic Search Examples
<!-- type: doc lang: markdown -->

### Example 6: Find All Function Usages

```rust
use cclab-lens::types::{SemanticSearchEngine, SearchQuery, SearchKind, SearchScope};

fn find_function_usages() -> Result<(), Box<dyn std::error::Error>> {
    let engine = SemanticSearchEngine::new();

    let query = SearchQuery {
        kind: SearchKind::Usages {
            symbol: "calculate_total".to_string(),
            file: PathBuf::from("billing.py"),
        },
        scope: SearchScope::Project,
        max_results: 100,
    };

    let result = engine.search(&query);

    println!("Found {} usages of 'calculate_total':", result.total_count);
    for (i, match_item) in result.matches.iter().enumerate() {
        println!("{}. {}:{}-{}",
            i + 1,
            match_item.file.display(),
            match_item.span.start_line,
            match_item.span.end_line
        );

        if let Some(context) = &match_item.context {
            println!("   {}", context.trim());
        }
    }

    Ok(())
}
```

**Output**:
```
Found 5 usages of 'calculate_total':
1. billing.py:15-15
   total = calculate_total(items)
2. checkout.py:42-42
   amount = calculate_total(cart.items)
3. invoice.py:78-78
   invoice_total = calculate_total(line_items)
...
```

---

### Example 7: Search by Type Signature

```rust
fn find_validation_functions() -> Result<(), Box<dyn std::error::Error>> {
    let engine = SemanticSearchEngine::new();

    // Find all (str) -> bool functions
    let query = SearchQuery {
        kind: SearchKind::ByTypeSignature {
            params: vec![Type::Str],
            return_type: Some(Type::Bool),
        },
        scope: SearchScope::Directory(PathBuf::from("src/validators")),
        max_results: 50,
    };

    let result = engine.search(&query);

    println!("Validation functions found:");
    for match_item in result.matches {
        if let Some(symbol) = match_item.symbol {
            println!("- {} in {}", symbol, match_item.file.display());
        }
    }

    Ok(())
}
```

**Output**:
```
Validation functions found:
- is_valid_email in src/validators/email.py
- is_valid_phone in src/validators/phone.py
- is_valid_username in src/validators/user.py
```

---

### Example 8: Find Call Hierarchy

```rust
fn analyze_call_chain() -> Result<(), Box<dyn std::error::Error>> {
    let engine = SemanticSearchEngine::new();

    // Find who calls this function
    let query = SearchQuery {
        kind: SearchKind::CallHierarchy {
            symbol: "send_email".to_string(),
            file: PathBuf::from("notifications.py"),
            direction: CallDirection::Callers,
        },
        scope: SearchScope::Project,
        max_results: 50,
    };

    let result = engine.search(&query);

    println!("Functions calling 'send_email':");
    for match_item in result.matches {
        if let Some(caller) = match_item.symbol {
            println!("  {} ({})", caller, match_item.file.display());
        }
    }

    // Find what this function calls
    let query2 = SearchQuery {
        kind: SearchKind::CallHierarchy {
            symbol: "send_email".to_string(),
            file: PathBuf::from("notifications.py"),
            direction: CallDirection::Callees,
        },
        scope: SearchScope::Project,
        max_results: 50,
    };

    let result2 = engine.search(&query2);

    println!("\nFunctions called by 'send_email':");
    for match_item in result2.matches {
        if let Some(callee) = match_item.symbol {
            println!("  {} ({})", callee, match_item.file.display());
        }
    }

    Ok(())
}
```

**Output**:
```
Functions calling 'send_email':
  notify_user (user_service.py)
  send_welcome (onboarding.py)
  alert_admin (admin_service.py)

Functions called by 'send_email':
  validate_email (validators.py)
  format_message (formatters.py)
  smtp_send (email_client.py)
```

---

### Example 9: Find Implementation of Protocol

```rust
fn find_serializable_classes() -> Result<(), Box<dyn std::error::Error>> {
    let engine = SemanticSearchEngine::new();

    let query = SearchQuery {
        kind: SearchKind::Implementations {
            protocol: "Serializable".to_string(),
        },
        scope: SearchScope::Project,
        max_results: 100,
    };

    let result = engine.search(&query);

    println!("Classes implementing Serializable:");
    for match_item in result.matches {
        if let Some(class_name) = match_item.symbol {
            println!("- {}", class_name);
        }
    }

    Ok(())
}
```

**Output**:
```
Classes implementing Serializable:
- User
- Product
- Order
- Invoice
- Customer
```

---

## Framework Support Examples
<!-- type: doc lang: markdown -->

### Example 10: Django Model Field Renaming

```python
# Original Django model
from django.db import models

class User(models.Model):
    username = models.CharField(max_length=100)
    email_address = models.EmailField()
```

```rust
// Rename email_address -> email
let request = RefactorRequest {
    kind: RefactorKind::Rename {
        new_name: "email".to_string(),
    },
    file: PathBuf::from("models.py"),
    span: Span::new(95, 109), // "email_address"
    options: Default::default(),
};

let mut engine = RefactoringEngine::new();
let result = engine.execute(&request, source);
```

**Output**:
```python
class User(models.Model):
    username = models.CharField(max_length=100)
    email = models.EmailField()  # Renamed
```

**Future Enhancement**: Auto-update QuerySet calls:
```python
# Would also update:
User.objects.filter(email_address="...")
# → User.objects.filter(email="...")
```

---

### Example 11: FastAPI Route Refactoring

```python
# Original FastAPI route
from fastapi import FastAPI

app = FastAPI()

@app.get("/users/{user_id}")
def get_user(user_id: int):
    return {"id": user_id}
```

```rust
// Extract validation logic
let request = RefactorRequest {
    kind: RefactorKind::ExtractFunction {
        name: "validate_user_id".to_string(),
    },
    file: PathBuf::from("routes.py"),
    span: Span::new(/* validation code */),
    options: Default::default(),
};
```

---

## Integration Workflows
<!-- type: doc lang: markdown -->

### Example 12: Safe Rename Workflow

```rust
fn safe_rename_workflow(
    symbol: &str,
    new_name: &str,
    file: &Path,
    source: &str
) -> Result<String, Box<dyn std::error::Error>> {
    // Step 1: Search for all usages
    let search_engine = SemanticSearchEngine::new();
    let usages_query = SearchQuery {
        kind: SearchKind::Usages {
            symbol: symbol.to_string(),
            file: file.to_path_buf(),
        },
        scope: SearchScope::Project,
        max_results: 1000,
    };

    let usages = search_engine.search(&usages_query);
    println!("Found {} usages", usages.total_count);

    // Step 2: Check for conflicts
    let conflict_query = SearchQuery {
        kind: SearchKind::Usages {
            symbol: new_name.to_string(),
            file: file.to_path_buf(),
        },
        scope: SearchScope::Project,
        max_results: 10,
    };

    let conflicts = search_engine.search(&conflict_query);
    if !conflicts.matches.is_empty() {
        return Err(format!("Name '{}' already exists!", new_name).into());
    }

    // Step 3: Perform rename
    let mut refactor_engine = RefactoringEngine::new();
    let rename_request = RefactorRequest {
        kind: RefactorKind::Rename {
            new_name: new_name.to_string(),
        },
        file: file.to_path_buf(),
        span: /* symbol span */,
        options: Default::default(),
    };

    let result = refactor_engine.execute(&rename_request, source);

    if result.has_errors() {
        return Err("Refactoring failed".into());
    }

    // Step 4: Apply edits
    Ok(apply_edits(source, result.file_edits))
}
```

---

### Example 13: Extract and Test Workflow

```rust
fn extract_and_verify() -> Result<(), Box<dyn std::error::Error>> {
    let source = read_file("complex.py")?;

    // Step 1: Extract function
    let extract_req = RefactorRequest {
        kind: RefactorKind::ExtractFunction {
            name: "calculate_score".to_string(),
        },
        file: PathBuf::from("complex.py"),
        span: Span::new(100, 250),
        options: Default::default(),
    };

    let mut engine = RefactoringEngine::new();
    let extract_result = engine.execute(&extract_req, &source);

    let modified = apply_edits(&source, extract_result.file_edits);

    // Step 2: Write to temp file
    std::fs::write("complex_temp.py", &modified)?;

    // Step 3: Run tests
    let test_output = std::process::Command::new("pytest")
        .arg("tests/test_complex.py")
        .output()?;

    if !test_output.status.success() {
        eprintln!("Tests failed! Rolling back...");
        return Err("Extraction broke tests".into());
    }

    // Step 4: Apply if tests pass
    std::fs::write("complex.py", &modified)?;
    println!("Extraction successful and tested!");

    Ok(())
}
```

---

### Example 14: Batch Refactoring

```rust
fn batch_rename_variables() -> Result<(), Box<dyn std::error::Error>> {
    let renames = vec![
        ("old_var1", "new_var1"),
        ("old_var2", "new_var2"),
        ("old_var3", "new_var3"),
    ];

    let mut engine = RefactoringEngine::new();
    let source = read_file("code.py")?;
    let mut current_source = source.clone();

    for (old_name, new_name) in renames {
        println!("Renaming {} -> {}", old_name, new_name);

        let request = RefactorRequest {
            kind: RefactorKind::Rename {
                new_name: new_name.to_string(),
            },
            file: PathBuf::from("code.py"),
            span: find_symbol_span(&current_source, old_name)?,
            options: Default::default(),
        };

        let result = engine.execute(&request, &current_source);

        if result.has_errors() {
            eprintln!("Failed to rename {}", old_name);
            continue;
        }

        current_source = apply_edits(&current_source, result.file_edits);
    }

    std::fs::write("code.py", current_source)?;
    println!("Batch rename complete!");

    Ok(())
}
```

---

## Real-World Scenarios
<!-- type: doc lang: markdown -->

### Scenario 1: Refactoring Legacy Code

```rust
// Problem: Large function with multiple responsibilities
// Solution: Extract smaller functions

fn refactor_legacy_function() -> Result<(), Box<dyn std::error::Error>> {
    let source = read_file("legacy.py")?;
    let mut engine = RefactoringEngine::new();

    // Extract validation logic
    let validation_req = RefactorRequest {
        kind: RefactorKind::ExtractFunction {
            name: "validate_input".to_string(),
        },
        file: PathBuf::from("legacy.py"),
        span: Span::new(50, 150), // Validation block
        options: Default::default(),
    };

    let step1 = engine.execute(&validation_req, &source);
    let source2 = apply_edits(&source, step1.file_edits);

    // Extract processing logic
    let processing_req = RefactorRequest {
        kind: RefactorKind::ExtractFunction {
            name: "process_data".to_string(),
        },
        file: PathBuf::from("legacy.py"),
        span: Span::new(200, 350), // Processing block
        options: Default::default(),
    };

    let step2 = engine.execute(&processing_req, &source2);
    let final_source = apply_edits(&source2, step2.file_edits);

    std::fs::write("legacy.py", final_source)?;
    println!("Refactoring complete! Function split into smaller parts.");

    Ok(())
}
```

---

### Scenario 2: API Modernization

```rust
// Problem: Deprecated parameter names
// Solution: Bulk rename with signature changes

fn modernize_api() -> Result<(), Box<dyn std::error::Error>> {
    let source = read_file("api.py")?;
    let mut engine = RefactoringEngine::new();

    // Change old parameter names
    let changes = SignatureChanges {
        new_params: vec![
            ("user_id".to_string(), Some("int".to_string()), None),
            // Old: usr_id
        ],
        param_order: vec![],
        removed_params: vec!["usr_id".to_string()],
        new_return_type: None,
    };

    let request = RefactorRequest {
        kind: RefactorKind::ChangeSignature { changes },
        file: PathBuf::from("api.py"),
        span: Span::new(/* function signature */),
        options: Default::default(),
    };

    let result = engine.execute(&request, &source);
    let modified = apply_edits(&source, result.file_edits);

    std::fs::write("api.py", modified)?;
    println!("API modernized!");

    Ok(())
}
```

---

## Error Handling Patterns
<!-- type: doc lang: markdown -->

### Pattern 1: Graceful Degradation

```rust
fn safe_refactor(request: RefactorRequest, source: &str) -> String {
    let mut engine = RefactoringEngine::new();
    let result = engine.execute(&request, source);

    if result.has_errors() {
        eprintln!("Refactoring failed, returning original source");
        for diag in result.diagnostics {
            eprintln!("  {}: {}", diag.level, diag.message);
        }
        return source.to_string();
    }

    if !result.has_changes() {
        println!("No changes needed");
        return source.to_string();
    }

    apply_edits(source, result.file_edits)
}
```

### Pattern 2: Transaction-Style Refactoring

```rust
fn transactional_refactor() -> Result<(), Box<dyn std::error::Error>> {
    let files = vec!["file1.py", "file2.py", "file3.py"];
    let mut backups = HashMap::new();

    // Backup all files
    for file in &files {
        backups.insert(file, read_file(file)?);
    }

    // Attempt refactorings
    match apply_refactorings(&files) {
        Ok(_) => {
            println!("All refactorings successful!");
            Ok(())
        }
        Err(e) => {
            eprintln!("Refactoring failed: {}. Rolling back...", e);

            // Restore backups
            for (file, content) in backups {
                std::fs::write(file, content)?;
            }

            Err(e)
        }
    }
}
```

---

## See Also
<!-- type: doc lang: markdown -->

- [Refactoring API Documentation](./REFACTORING_API.md)
- [Semantic Search API Documentation](./SEMANTIC_SEARCH_API.md)
- [Framework Support Guide](./FRAMEWORK_SUPPORT.md)
- [Integration Tests](../crates/cclab-lens/tests/test_p0_integration.rs)

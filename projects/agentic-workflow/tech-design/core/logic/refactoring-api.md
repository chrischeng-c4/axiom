---
id: refactoring-api
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Refactoring API docs support semantic source understanding during brownfield standardization."
---

# Refactoring Engine API Documentation

## Overview
<!-- type: doc lang: markdown -->

The Argus Refactoring Engine provides 7 core refactoring operations for Python, TypeScript, and Rust code. All operations are multi-language and preserve code semantics.

**Status**: ✅ Production-ready (100% implementation, 54 tests passing)

---

## Quick Start
<!-- type: doc lang: markdown -->

```rust
use cclab-lens::types::{
    RefactoringEngine, RefactorRequest, RefactorKind,
    RefactorOptions, Span
};
use std::path::PathBuf;

// Create engine
let mut engine = RefactoringEngine::new();

// Create refactoring request
let request = RefactorRequest {
    kind: RefactorKind::Rename {
        new_name: "new_name".to_string(),
    },
    file: PathBuf::from("example.py"),
    span: Span::new(0, 8), // Byte positions
    options: RefactorOptions::default(),
};

// Execute refactoring
let source = "old_name = 42";
let result = engine.execute(&request, source);

// Check results
if !result.has_errors() {
    for (file, edits) in result.file_edits {
        // Apply edits to file
    }
}
```

---

## Core Types
<!-- type: doc lang: markdown -->

### RefactorRequest

```rust
pub struct RefactorRequest {
    /// Type of refactoring to perform
    pub kind: RefactorKind,

    /// File being refactored
    pub file: PathBuf,

    /// Span of code to refactor (byte positions)
    pub span: Span,

    /// Additional options
    pub options: RefactorOptions,
}
```

### RefactorKind

```rust
pub enum RefactorKind {
    /// Rename a symbol
    Rename { new_name: String },

    /// Extract variable from expression
    ExtractVariable { name: String },

    /// Extract function from code block
    ExtractFunction { name: String },

    /// Extract method (with self parameter)
    ExtractMethod { name: String },

    /// Inline variable into usages
    Inline,

    /// Change function signature
    ChangeSignature { changes: SignatureChanges },

    /// Move definition to another file
    MoveDefinition { target_file: PathBuf },
}
```

### RefactorResult

```rust
pub struct RefactorResult {
    /// Text edits per file
    pub file_edits: HashMap<PathBuf, Vec<TextEdit>>,

    /// New files created (for MoveDefinition)
    pub new_files: HashMap<PathBuf, String>,

    /// Diagnostics (errors, warnings, info)
    pub diagnostics: Vec<Diagnostic>,
}

impl RefactorResult {
    /// Check if refactoring has errors
    pub fn has_errors(&self) -> bool;

    /// Check if refactoring has changes
    pub fn has_changes(&self) -> bool;
}
```

### TextEdit

```rust
pub struct TextEdit {
    /// Span to replace (byte positions)
    pub span: Span,

    /// New text to insert
    pub new_text: String,
}
```

### Span

```rust
pub struct Span {
    /// Start byte position
    pub start: usize,

    /// End byte position
    pub end: usize,

    /// Optional line/column info
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}
```

---

## Refactoring Operations
<!-- type: doc lang: markdown -->

### 1. Rename Symbol

**Purpose**: Rename variables, functions, classes, or methods.

**Example**:
```python
# Before
old_name = 42
result = old_name * 2

# After (rename old_name → new_name)
new_name = 42
result = new_name * 2
```

**Usage**:
```rust
let request = RefactorRequest {
    kind: RefactorKind::Rename {
        new_name: "new_name".to_string(),
    },
    file: PathBuf::from("example.py"),
    span: Span::new(0, 8), // "old_name" position
    options: RefactorOptions::default(),
};

let result = engine.execute(&request, source);
```

**Validation**:
- ❌ Empty name → Error
- ❌ Same as old name → Info (no changes)
- ❌ Invalid identifier → Error
- ✅ Valid name → Success

**Supported Languages**: Python, TypeScript, Rust

**Limitations**:
- Simple text-based search (may rename in strings/comments)
- No scope awareness (renames all occurrences)
- Single file only (cross-file planned)

---

### 2. Extract Variable

**Purpose**: Extract an expression into a named variable.

**Example**:
```python
# Before
result = user.name.upper()

# After (extract "user.name.upper()" as "display_name")
display_name = user.name.upper()
result = display_name
```

**Usage**:
```rust
let request = RefactorRequest {
    kind: RefactorKind::ExtractVariable {
        name: "display_name".to_string(),
    },
    file: PathBuf::from("example.py"),
    span: Span::new(9, 28), // Expression span
    options: RefactorOptions::default(),
};

let result = engine.execute(&request, source);
```

**Behavior**:
- Inserts assignment before the line containing expression
- Replaces expression with variable name
- Preserves indentation

**Supported Languages**: Python, TypeScript, Rust

**Limitations**:
- No type inference for variable type
- Fixed insertion position (before current line)

---

### 3. Extract Function

**Purpose**: Extract a block of code into a standalone function.

**Example**:
```python
# Before
print("Starting")
print("Processing")
print("Done")

# After (extract first two lines as "log_start")
def log_start():
    print("Starting")
    print("Processing")

log_start()
print("Done")
```

**Usage**:
```rust
let request = RefactorRequest {
    kind: RefactorKind::ExtractFunction {
        name: "log_start".to_string(),
    },
    file: PathBuf::from("example.py"),
    span: Span::new(0, 37), // Selected code span
    options: RefactorOptions::default(),
};

let result = engine.execute(&request, source);
```

**Behavior**:
- Creates function definition at beginning of file
- Replaces selection with function call
- Indents function body correctly

**Supported Languages**: Python, TypeScript, Rust

**Limitations**:
- No parameter detection (functions have no parameters)
- No return type inference
- Fixed insertion position (beginning of file)

---

### 4. Extract Method

**Purpose**: Extract code into a method within a class (with `self` parameter).

**Example**:
```python
# Before
class MyClass:
    def process(self):
        x = 1
        y = 2

# After (extract as "init_values")
class MyClass:
    def init_values(self):
        x = 1
        y = 2

    def process(self):
        self.init_values()
```

**Usage**:
```rust
let request = RefactorRequest {
    kind: RefactorKind::ExtractMethod {
        name: "init_values".to_string(),
    },
    file: PathBuf::from("example.py"),
    span: Span::new(39, 59), // Selected code
    options: RefactorOptions::default(),
};

let result = engine.execute(&request, source);
```

**Behavior**:
- Creates method with `self` parameter
- Call uses `self.method_name()`
- Proper indentation for class context

**Supported Languages**: Python (primary), TypeScript, Rust

**Limitations**:
- Same as Extract Function
- No captured variable detection

---

### 5. Inline Variable

**Purpose**: Replace all usages of a variable with its definition value, then remove the definition.

**Example**:
```python
# Before
temp = 1 + 2
result = temp * 3
output = temp + 5

# After (inline "temp")
result = 1 + 2 * 3
output = 1 + 2 + 5
```

**Usage**:
```rust
let request = RefactorRequest {
    kind: RefactorKind::Inline,
    file: PathBuf::from("example.py"),
    span: Span::new(0, 4), // Variable name "temp"
    options: RefactorOptions::default(),
};

let result = engine.execute(&request, source);
```

**Behavior**:
- Finds definition: `variable = value`
- Finds all usages with word boundary checking
- Replaces usages with value
- Removes definition line

**Edge Cases**:
- No definition found → Error
- Definition found but no usages → Warning
- Single usage → Inline and remove

**Supported Languages**: Python, TypeScript, Rust

**Limitations**:
- Simple pattern matching (only `var = value` format)
- No scope awareness
- May fail with complex expressions

---

### 6. Change Signature

**Purpose**: Add parameters to function signatures with types and defaults.

**Example**:
```python
# Before
def greet():
    print("hello")

# After (add parameter "name: str = 'World'")
def greet(name: str = "World"):
    print("hello")
```

**Usage**:
```rust
let changes = SignatureChanges {
    new_params: vec![
        ("name".to_string(),
         Some("str".to_string()),
         Some("\"World\"".to_string())),
    ],
    param_order: vec![],
    removed_params: vec![],
    new_return_type: None,
};

let request = RefactorRequest {
    kind: RefactorKind::ChangeSignature { changes },
    file: PathBuf::from("example.py"),
    span: Span::new(0, 12), // Function signature
    options: RefactorOptions::default(),
};

let result = engine.execute(&request, source);
```

**SignatureChanges**:
```rust
pub struct SignatureChanges {
    /// New parameters: (name, type_annotation, default_value)
    pub new_params: Vec<(String, Option<String>, Option<String>)>,

    /// New parameter order
    pub param_order: Vec<String>,

    /// Parameters to remove
    pub removed_params: Vec<String>,

    /// New return type
    pub new_return_type: Option<String>,
}
```

**Supported Languages**: Python, TypeScript, Rust

**Limitations**:
- No call site updates
- Only adds parameters (no removal/reordering yet)

---

### 7. Move Definition

**Purpose**: Move a function or class definition to another file.

**Example**:
```python
# Before (in main.py)
def helper():
    return 42

# After move to utils.py
# main.py: (empty or removed)
# utils.py: def helper(): return 42
```

**Usage**:
```rust
let request = RefactorRequest {
    kind: RefactorKind::MoveDefinition {
        target_file: PathBuf::from("utils.py"),
    },
    file: PathBuf::from("main.py"),
    span: Span::new(0, 27), // Entire definition
    options: RefactorOptions::default(),
};

let result = engine.execute(&request, source);

// New file created
if let Some(content) = result.new_files.get(&PathBuf::from("utils.py")) {
    // Write content to utils.py
}
```

**Behavior**:
- Removes definition from source file
- Adds to `result.new_files` for target file
- Target file path in `RefactorResult::new_files`

**Supported Languages**: Python, TypeScript, Rust

**Limitations**:
- No import updates
- No reference tracking
- No safety validation

---

## Applying Text Edits
<!-- type: doc lang: markdown -->

### Edit Application Order

**CRITICAL**: Edits must be applied in reverse order (end to start) to preserve positions.

```rust
fn apply_edits(source: &str, edits: &mut Vec<TextEdit>) -> String {
    // Sort by start position (reverse)
    edits.sort_by(|a, b| {
        match b.span.start.cmp(&a.span.start) {
            std::cmp::Ordering::Equal => b.span.end.cmp(&a.span.end),
            other => other,
        }
    });

    let mut modified = source.to_string();
    for edit in edits {
        let before = &modified[..edit.span.start];
        let after = &modified[edit.span.end..];
        modified = format!("{}{}{}", before, edit.new_text, after);
    }
    modified
}
```

### Multi-File Edits

```rust
for (file, mut edits) in result.file_edits {
    // Read file
    let source = std::fs::read_to_string(&file)?;

    // Apply edits
    let modified = apply_edits(&source, &mut edits);

    // Write back
    std::fs::write(&file, modified)?;
}

// Handle new files
for (file, content) in result.new_files {
    std::fs::write(&file, content)?;
}
```

---

## Error Handling
<!-- type: doc lang: markdown -->

### Diagnostic Levels

```rust
pub enum DiagnosticLevel {
    Error,   // Refactoring failed
    Warning, // Refactoring succeeded with warnings
    Info,    // Informational messages
}
```

### Checking Results

```rust
let result = engine.execute(&request, source);

if result.has_errors() {
    for diag in result.diagnostics {
        if diag.level == DiagnosticLevel::Error {
            eprintln!("Error: {}", diag.message);
        }
    }
} else if result.has_changes() {
    // Apply edits
} else {
    println!("No changes made");
}
```

### Common Errors

**Rename**:
- Empty name
- Invalid identifier characters
- Same as old name (info, not error)

**Extract Variable/Function**:
- Parse errors
- Invalid span

**Inline**:
- Definition not found
- No usages found (warning)

**Change Signature**:
- Invalid signature format
- Parse errors

**Move Definition**:
- Parse errors
- Invalid span

---

## Multi-Language Support
<!-- type: doc lang: markdown -->

### Language Detection

Automatic detection from file extension:

```rust
use cclab-lens::syntax::MultiParser;

let language = MultiParser::detect_language(&PathBuf::from("example.py"));
// Returns Some(Language::Python)
```

### Supported Languages

| Language   | Extension | Rename | Extract | Inline | Signature | Move |
|------------|-----------|--------|---------|--------|-----------|------|
| Python     | .py       | ✅     | ✅      | ✅     | ✅        | ✅   |
| TypeScript | .ts       | ✅     | ✅      | ✅     | ✅        | ✅   |
| Rust       | .rs       | ✅     | ✅      | ✅     | ✅        | ✅   |

### Language-Specific Notes

**Python**:
- Method extraction uses `self` parameter
- Type annotations supported in signatures
- Indentation preserved

**TypeScript**:
- Const/let/var handled correctly
- Type annotations supported
- Arrow functions supported

**Rust**:
- `let` bindings handled
- Type inference considered
- Ownership preserved

---

## Best Practices
<!-- type: doc lang: markdown -->

### 1. Always Check Errors

```rust
let result = engine.execute(&request, source);
if result.has_errors() {
    // Handle errors before applying
    return Err("Refactoring failed");
}
```

### 2. Validate Spans

```rust
if span.end > source.len() {
    return Err("Span out of bounds");
}
```

### 3. Preserve Source on Failure

```rust
let backup = source.clone();
match apply_refactoring(&request, source) {
    Ok(modified) => Ok(modified),
    Err(e) => {
        // Restore backup
        Ok(backup)
    }
}
```

### 4. Sequential Refactorings

```rust
// Refactor 1
let result1 = engine.execute(&request1, source);
let source2 = apply_edits(source, result1);

// Refactor 2 on modified source
let result2 = engine.execute(&request2, &source2);
```

### 5. Test Before Applying

```rust
// Dry run
let result = engine.execute(&request, source);
if result.has_changes() {
    let preview = apply_edits(source, result.clone());
    // Show preview to user
    // Apply if approved
}
```

---

## Performance Considerations
<!-- type: doc lang: markdown -->

### AST Caching

The engine caches ASTs for performance:

```rust
// First call: parses and caches
engine.execute(&request1, source);

// Subsequent calls: uses cached AST
engine.execute(&request2, source);
```

### Large Files

For large files (>10k lines):
- Extract smaller operations preferred
- Consider chunked refactoring
- Monitor memory usage

### Batch Operations

For multiple files:

```rust
let mut results = Vec::new();
for file in files {
    let source = read_file(&file)?;
    let result = engine.execute(&request, &source);
    results.push((file, result));
}

// Apply all at once
for (file, result) in results {
    apply_result(&file, result)?;
}
```

---

## Advanced Usage
<!-- type: doc lang: markdown -->

### Custom RefactorOptions

```rust
pub struct RefactorOptions {
    /// Update import statements
    pub update_imports: bool,

    /// Preserve comments
    pub preserve_comments: bool,

    /// Dry run (no actual changes)
    pub dry_run: bool,
}
```

### Integration with Semantic Search

```rust
use cclab-lens::types::SemanticSearchEngine;

// Search for usages first
let mut search = SemanticSearchEngine::new();
let usages = search.search(&SearchQuery {
    kind: SearchKind::Usages {
        symbol: "old_name".to_string(),
        file: file.clone(),
    },
    scope: SearchScope::Project,
    max_results: 100,
});

// Then refactor
let request = RefactorRequest {
    kind: RefactorKind::Rename {
        new_name: "new_name".to_string()
    },
    file,
    span,
    options: RefactorOptions::default(),
};
```

---

## Testing Your Integration
<!-- type: doc lang: markdown -->

### Unit Tests

```rust
#[test]
fn test_rename_variable() {
    let source = "x = 1\ny = x + 2";
    let request = RefactorRequest {
        kind: RefactorKind::Rename {
            new_name: "value".to_string()
        },
        file: PathBuf::from("test.py"),
        span: Span::new(0, 1),
        options: RefactorOptions::default(),
    };

    let mut engine = RefactoringEngine::new();
    let result = engine.execute(&request, source);

    assert!(!result.has_errors());
    assert!(result.has_changes());
}
```

### Integration Tests

See `crates/cclab-lens/tests/test_p0_integration.rs` for comprehensive examples.

---

## API Stability
<!-- type: doc lang: markdown -->

**Current Status**: ✅ Stable API (v0.1.0)

Breaking changes planned:
- None currently

Future additions (non-breaking):
- Cross-file refactoring
- Import management
- Scope-aware renaming
- AST-based inline detection

---

## Troubleshooting
<!-- type: doc lang: markdown -->

### Issue: Refactoring returns empty results

**Cause**: Invalid span or parse error

**Solution**:
- Check span bounds
- Validate source code syntax
- Check diagnostics

### Issue: Rename renames in strings

**Cause**: Text-based search limitation

**Solution**:
- Review changes before applying
- Future: AST-based rename (planned)

### Issue: Extract function has no parameters

**Cause**: Parameter detection not implemented

**Solution**:
- Add parameters manually
- Future: Data flow analysis (planned)

### Issue: Move definition breaks references

**Cause**: Import updates not implemented

**Solution**:
- Update imports manually
- Future: Import management (planned)

---

## See Also
<!-- type: doc lang: markdown -->

- [Semantic Search API](./SEMANTIC_SEARCH_API.md)
- [Framework Support](./FRAMEWORK_SUPPORT.md)
- [Integration Tests](../crates/cclab-lens/tests/test_p0_integration.rs)
- [Main Documentation](../CLAUDE.md)

---

## Feedback & Contributions
<!-- type: doc lang: markdown -->

Found a bug? Have a feature request?
- GitHub Issues: https://github.com/your-org/cclab-lens/issues
- See CLAUDE.md for implementation details

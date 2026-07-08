---
id: libs-compass-src-type-inference-codegen-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/type_inference/codegen.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/type_inference/codegen.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/type_inference/codegen.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CodeGenRequest` | libs/compass/src/type_inference/codegen.rs | struct | pub | 19 | pub struct CodeGenRequest { |
| `CodeGenKind` | libs/compass/src/type_inference/codegen.rs | enum | pub | 32 | pub enum CodeGenKind { |
| `DocstringStyle` | libs/compass/src/type_inference/codegen.rs | enum | pub | 51 | pub enum DocstringStyle { |
| `TestFramework` | libs/compass/src/type_inference/codegen.rs | enum | pub | 64 | pub enum TestFramework { |
| `CodeGenOptions` | libs/compass/src/type_inference/codegen.rs | struct | pub | 75 | pub struct CodeGenOptions { |
| `CodeGenResult` | libs/compass/src/type_inference/codegen.rs | struct | pub | 92 | pub struct CodeGenResult { |
| `new` | libs/compass/src/type_inference/codegen.rs | function | pub | 105 | pub fn new(code: impl Into<String>, target_file: PathBuf) -> Self { |
| `at_line` | libs/compass/src/type_inference/codegen.rs | function | pub | 115 | pub fn at_line(mut self, line: usize) -> Self { |
| `with_import` | libs/compass/src/type_inference/codegen.rs | function | pub | 121 | pub fn with_import(mut self, import: impl Into<String>) -> Self { |
| `CodeGenerator` | libs/compass/src/type_inference/codegen.rs | struct | pub | 132 | pub struct CodeGenerator { |
| `with_context` | libs/compass/src/type_inference/codegen.rs | function | pub | 154 | pub fn with_context(type_context: TypeContext) -> Self { |
| `generate` | libs/compass/src/type_inference/codegen.rs | function | pub | 162 | pub fn generate(&self, request: &CodeGenRequest) -> CodeGenResult { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Code generation (Sprint 3 - Track 2)
//!
//! Provides type-aware code generation:
//! - Docstring generation from types
//! - Test stub generation
//! - Type stub (pyi) generation
//! - Implementation from interface

use std::path::PathBuf;

use super::deep_inference::TypeContext;

// ============================================================================
// Code Generation Request
// ============================================================================

/// A request for code generation.
#[derive(Debug, Clone)]
pub struct CodeGenRequest {
    /// Type of generation
    pub kind: CodeGenKind,
    /// Source file
    pub file: PathBuf,
    /// Target symbol (function, class, etc.)
    pub symbol: String,
    /// Generation options
    pub options: CodeGenOptions,
}

/// Type of code generation.
#[derive(Debug, Clone)]
pub enum CodeGenKind {
    /// Generate docstring
    Docstring { style: DocstringStyle },
    /// Generate test stubs
    TestStub { framework: TestFramework },
    /// Generate type stub (.pyi) for a symbol
    TypeStub,
    /// Generate type stub for entire module
    ModuleStub,
    /// Generate implementation from protocol/ABC
    Implementation { protocol: String },
    /// Generate constructor (__init__)
    Constructor,
    /// Generate property accessors
    Properties { fields: Vec<String> },
}

/// Docstring style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocstringStyle {
    /// Google style
    Google,
    /// NumPy style
    NumPy,
    /// Sphinx style
    Sphinx,
    /// reStructuredText
    RST,
}

/// Test framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestFramework {
    /// pytest
    Pytest,
    /// unittest
    Unittest,
    /// doctest
    Doctest,
}

/// Options for code generation.
#[derive(Debug, Clone, Default)]
pub struct CodeGenOptions {
    /// Include type annotations
    pub include_types: bool,
    /// Include examples in docstrings
    pub include_examples: bool,
    /// Generate async versions
    pub async_support: bool,
    /// Indentation (spaces)
    pub indent: usize,
}

// ============================================================================
// Code Generation Result
// ============================================================================

/// Result of code generation.
#[derive(Debug, Clone)]
pub struct CodeGenResult {
    /// Generated code
    pub code: String,
    /// File to insert into
    pub target_file: PathBuf,
    /// Position to insert (line number)
    pub insert_line: usize,
    /// Imports needed
    pub imports: Vec<String>,
}

impl CodeGenResult {
    /// Create a new result.
    pub fn new(code: impl Into<String>, target_file: PathBuf) -> Self {
        Self {
            code: code.into(),
            target_file,
            insert_line: 0,
            imports: Vec::new(),
        }
    }

    /// Set insertion line.
    pub fn at_line(mut self, line: usize) -> Self {
        self.insert_line = line;
        self
    }

    /// Add required import.
    pub fn with_import(mut self, import: impl Into<String>) -> Self {
        self.imports.push(import.into());
        self
    }
}

// ============================================================================
// Code Generator
// ============================================================================

/// Code generator with type awareness.
pub struct CodeGenerator {
    /// Type context for type information
    type_context: TypeContext,
    /// Default options
    default_options: CodeGenOptions,
}

impl CodeGenerator {
    /// Create a new code generator.
    pub fn new() -> Self {
        Self {
            type_context: TypeContext::new(),
            default_options: CodeGenOptions {
                include_types: true,
                include_examples: true,
                async_support: false,
                indent: 4,
            },
        }
    }

    /// Create with type context.
    pub fn with_context(type_context: TypeContext) -> Self {
        Self {
            type_context,
            default_options: CodeGenOptions::default(),
        }
    }

    /// Generate code based on request.
    pub fn generate(&self, request: &CodeGenRequest) -> CodeGenResult {
        match &request.kind {
            CodeGenKind::Docstring { style } => self.generate_docstring(request, *style),
            CodeGenKind::TestStub { framework } => self.generate_test_stub(request, *framework),
            CodeGenKind::TypeStub => self.generate_type_stub(request),
            CodeGenKind::ModuleStub => self.generate_module_stub(request),
            CodeGenKind::Implementation { protocol } => {
                self.generate_implementation(request, protocol)
            }
            CodeGenKind::Constructor => self.generate_constructor(request),
            CodeGenKind::Properties { fields } => self.generate_properties(request, fields),
        }
    }

    /// Generate docstring for a function/class.
    fn generate_docstring(&self, request: &CodeGenRequest, style: DocstringStyle) -> CodeGenResult {
        let indent = " ".repeat(request.options.indent.max(self.default_options.indent));

        let docstring = match style {
            DocstringStyle::Google => self.google_docstring(&request.symbol, &indent),
            DocstringStyle::NumPy => self.numpy_docstring(&request.symbol, &indent),
            DocstringStyle::Sphinx => self.sphinx_docstring(&request.symbol, &indent),
            DocstringStyle::RST => self.rst_docstring(&request.symbol, &indent),
        };

        CodeGenResult::new(docstring, request.file.clone())
    }

    fn google_docstring(&self, symbol: &str, indent: &str) -> String {
        format!(
            r#"{indent}"""Short description of {symbol}.

{indent}Longer description if needed.

{indent}Args:
{indent}    param1: Description of param1.
{indent}    param2: Description of param2.

{indent}Returns:
{indent}    Description of return value.

{indent}Raises:
{indent}    ValueError: If invalid input.
{indent}""""#,
            indent = indent,
            symbol = symbol
        )
    }

    fn numpy_docstring(&self, symbol: &str, indent: &str) -> String {
        format!(
            r#"{indent}"""
{indent}Short description of {symbol}.

{indent}Parameters
{indent}----------
{indent}param1 : type
{indent}    Description of param1.
{indent}param2 : type
{indent}    Description of param2.

{indent}Returns
{indent}-------
{indent}type
{indent}    Description of return value.
{indent}""""#,
            indent = indent,
            symbol = symbol
        )
    }

    fn sphinx_docstring(&self, symbol: &str, indent: &str) -> String {
        format!(
            r#"{indent}"""Short description of {symbol}.

{indent}:param param1: Description of param1.
{indent}:type param1: type
{indent}:param param2: Description of param2.
{indent}:type param2: type
{indent}:returns: Description of return value.
{indent}:rtype: type
{indent}:raises ValueError: If invalid input.
{indent}""""#,
            indent = indent,
            symbol = symbol
        )
    }

    fn rst_docstring(&self, symbol: &str, indent: &str) -> String {
        self.sphinx_docstring(symbol, indent)
    }

    /// Generate test stubs.
    fn generate_test_stub(
        &self,
        request: &CodeGenRequest,
        framework: TestFramework,
    ) -> CodeGenResult {
        let code = match framework {
            TestFramework::Pytest => self.pytest_stub(&request.symbol),
            TestFramework::Unittest => self.unittest_stub(&request.symbol),
            TestFramework::Doctest => self.doctest_stub(&request.symbol),
        };

        let mut result = CodeGenResult::new(code, request.file.clone());

        match framework {
            TestFramework::Pytest => {
                result = result.with_import("import pytest");
            }
            TestFramework::Unittest => {
                result = result.with_import("import unittest");
            }
            TestFramework::Doctest => {}
        }

        result
    }

    fn pytest_stub(&self, symbol: &str) -> String {
        format!(
            r#"import pytest


class Test{symbol}:
    """Tests for {symbol}."""

    def test_{symbol_lower}_basic(self):
        """Test basic functionality."""
        # Arrange

        # Act

        # Assert
        assert True

    def test_{symbol_lower}_edge_case(self):
        """Test edge cases."""
        # Arrange

        # Act

        # Assert
        assert True

    @pytest.mark.parametrize("input,expected", [
        ("input1", "expected1"),
        ("input2", "expected2"),
    ])
    def test_{symbol_lower}_parametrized(self, input, expected):
        """Test with various inputs."""
        assert True
"#,
            symbol = symbol,
            symbol_lower = symbol.to_lowercase()
        )
    }

    fn unittest_stub(&self, symbol: &str) -> String {
        format!(
            r#"import unittest


class Test{symbol}(unittest.TestCase):
    """Tests for {symbol}."""

    def setUp(self):
        """Set up test fixtures."""
        pass

    def tearDown(self):
        """Tear down test fixtures."""
        pass

    def test_{symbol_lower}_basic(self):
        """Test basic functionality."""
        self.assertTrue(True)

    def test_{symbol_lower}_edge_case(self):
        """Test edge cases."""
        self.assertTrue(True)


if __name__ == "__main__":
    unittest.main()
"#,
            symbol = symbol,
            symbol_lower = symbol.to_lowercase()
        )
    }

    fn doctest_stub(&self, symbol: &str) -> String {
        format!(
            r#"def {symbol_lower}():
    """
    Description of {symbol}.

    Examples
    --------
    >>> {symbol_lower}()
    expected_output

    >>> {symbol_lower}(arg)
    expected_output
    """
    pass
"#,
            symbol = symbol,
            symbol_lower = symbol.to_lowercase()
        )
    }

    /// Generate type stub (.pyi file).
    fn generate_type_stub(&self, request: &CodeGenRequest) -> CodeGenResult {
        use super::ty::Type;

        // Try to get type information from context
        let type_info = self
            .type_context
            .get_binding(&request.file, &request.symbol);

        let stub = match type_info.map(|b| &b.ty) {
            Some(Type::Callable { params, ret }) => {
                // Generate function stub with actual signature
                let params_str = params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, self.type_to_stub_annotation(&p.ty)))
                    .collect::<Vec<_>>()
                    .join(", ");

                let ret_str = self.type_to_stub_annotation(ret);

                format!(
                    r#"def {symbol}({params}) -> {ret}: ...
"#,
                    symbol = request.symbol,
                    params = params_str,
                    ret = ret_str
                )
            }
            Some(Type::ClassType { name, .. }) | Some(Type::Instance { name, .. }) => {
                // Generate class stub
                if let Some(class_info) = self.type_context.get_class_info(name) {
                    self.generate_class_stub(&request.symbol, class_info)
                } else {
                    // Fallback
                    format!(
                        r#"class {symbol}:
    """Class {symbol}."""
    ...
"#,
                        symbol = request.symbol
                    )
                }
            }
            _ => {
                // Fallback to generic signature
                format!(
                    r#"from typing import Any

def {symbol}(*args: Any, **kwargs: Any) -> Any: ...
"#,
                    symbol = request.symbol
                )
            }
        };

        let mut stub_file = request.file.clone();
        stub_file.set_extension("pyi");

        CodeGenResult::new(stub, stub_file)
    }

    /// Convert Type to stub annotation string.
    fn type_to_stub_annotation(&self, ty: &super::ty::Type) -> String {
        use super::ty::Type;

        match ty {
            Type::None => "None".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Int => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::Str => "str".to_string(),
            Type::Bytes => "bytes".to_string(),
            Type::List(inner) => format!("list[{}]", self.type_to_stub_annotation(inner)),
            Type::Dict(k, v) => format!(
                "dict[{}, {}]",
                self.type_to_stub_annotation(k),
                self.type_to_stub_annotation(v)
            ),
            Type::Set(inner) => format!("set[{}]", self.type_to_stub_annotation(inner)),
            Type::Tuple(types) => {
                let types_str = types
                    .iter()
                    .map(|t| self.type_to_stub_annotation(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("tuple[{}]", types_str)
            }
            Type::Optional(inner) => {
                format!("Optional[{}]", self.type_to_stub_annotation(inner))
            }
            Type::Union(types) => {
                let types_str = types
                    .iter()
                    .map(|t| self.type_to_stub_annotation(t))
                    .collect::<Vec<_>>()
                    .join(" | ");
                types_str
            }
            Type::Callable { params, ret } => {
                let params_str = params
                    .iter()
                    .map(|p| self.type_to_stub_annotation(&p.ty))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "Callable[[{}], {}]",
                    params_str,
                    self.type_to_stub_annotation(ret)
                )
            }
            Type::Instance {
                name, type_args, ..
            } => {
                if type_args.is_empty() {
                    name.clone()
                } else {
                    let args_str = type_args
                        .iter()
                        .map(|t| self.type_to_stub_annotation(t))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}[{}]", name, args_str)
                }
            }
            Type::ClassType { name, .. } => format!("type[{}]", name),
            Type::Any => "Any".to_string(),
            Type::Unknown => "Any".to_string(),
            _ => "Any".to_string(),
        }
    }

    /// Generate stub for a class.
    fn generate_class_stub(&self, name: &str, class_info: &super::class_info::ClassInfo) -> String {
        let mut stub = String::new();

        // Class declaration
        if class_info.bases.is_empty() {
            stub.push_str(&format!("class {}:\n", name));
        } else {
            let bases = class_info.bases.join(", ");
            stub.push_str(&format!("class {}({}):\n", name, bases));
        }

        // Docstring
        stub.push_str(&format!("    \"\"\"Class {}.\"\"\"\n\n", name));

        // Attributes
        for (attr_name, attr_type) in &class_info.attributes {
            stub.push_str(&format!(
                "    {}: {}\n",
                attr_name,
                self.type_to_stub_annotation(attr_type)
            ));
        }

        if !class_info.attributes.is_empty() {
            stub.push_str("\n");
        }

        // Methods
        for (method_name, method_type) in &class_info.methods {
            match method_type {
                super::ty::Type::Callable { params, ret } => {
                    let params_str = params
                        .iter()
                        .map(|p| format!("{}: {}", p.name, self.type_to_stub_annotation(&p.ty)))
                        .collect::<Vec<_>>()
                        .join(", ");

                    let ret_str = self.type_to_stub_annotation(ret);

                    stub.push_str(&format!(
                        "    def {}({}) -> {}: ...\n",
                        method_name, params_str, ret_str
                    ));
                }
                _ => {
                    stub.push_str(&format!("    def {}(self) -> Any: ...\n", method_name));
                }
            }
        }

        if class_info.methods.is_empty() && class_info.attributes.is_empty() {
            stub.push_str("    ...\n");
        }

        stub
    }

    /// Generate type stub for entire module (.pyi file).
    ///
    /// Note: This is a simplified implementation. For full module stub generation,
    /// we need access to all symbol bindings in the file, which requires passing
    /// FileAnalysis or extending TypeContext with a method to iterate bindings.
    fn generate_module_stub(&self, request: &CodeGenRequest) -> CodeGenResult {
        use super::ty::Type;
        use std::collections::HashSet;

        let mut stub = String::new();

        // Collect all imports needed
        let mut imports = HashSet::new();
        imports.insert("from typing import Any, Optional, Callable".to_string());

        // Generate stub for the requested symbol if available
        if let Some(binding) = self
            .type_context
            .get_binding(&request.file, &request.symbol)
        {
            let exported_symbols = vec![request.symbol.clone()];

            // Generate stub based on type
            match &binding.ty {
                Type::Callable { params, ret } => {
                    let params_str = params
                        .iter()
                        .map(|p| format!("{}: {}", p.name, self.type_to_stub_annotation(&p.ty)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let ret_str = self.type_to_stub_annotation(ret);
                    stub.push_str(&format!(
                        "\ndef {}({}) -> {}: ...\n",
                        request.symbol, params_str, ret_str
                    ));
                }
                Type::ClassType {
                    name: class_name, ..
                }
                | Type::Instance {
                    name: class_name, ..
                } => {
                    if let Some(class_info) = self.type_context.get_class_info(class_name) {
                        stub.push_str("\n");
                        stub.push_str(&self.generate_class_stub(&request.symbol, class_info));
                    } else {
                        stub.push_str(&format!("\nclass {}:\n    ...\n", request.symbol));
                    }
                }
                _ => {
                    stub.push_str(&format!("\n{}: Any\n", request.symbol));
                }
            }

            // Add __all__ export list
            let all_list = format!(
                "__all__ = [{}]\n\n",
                exported_symbols
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            stub.insert_str(0, &all_list);
        } else {
            // No binding found - generate minimal stub
            stub.push_str("# Module stub\n");
            stub.push_str(&format!("\n{}: Any\n", request.symbol));
        }

        // Prepend imports
        let imports_str = format!("{}\n\n", imports.into_iter().collect::<Vec<_>>().join("\n"));
        stub.insert_str(0, &imports_str);

        let mut stub_file = request.file.clone();
        stub_file.set_extension("pyi");

        CodeGenResult::new(stub, stub_file)
    }

    /// Generate implementation from protocol.
    fn generate_implementation(&self, request: &CodeGenRequest, protocol: &str) -> CodeGenResult {
        let code = format!(
            r#"class {symbol}({protocol}):
    """Implementation of {protocol}."""

    def __init__(self) -> None:
        """Initialize {symbol}."""
        pass

    # TODO: Implement required methods from {protocol}
"#,
            symbol = request.symbol,
            protocol = protocol
        );

        CodeGenResult::new(code, request.file.clone())
    }

    /// Generate constructor.
    fn generate_constructor(&self, request: &CodeGenRequest) -> CodeGenResult {
        let code = format!(
            r#"    def __init__(self) -> None:
        """Initialize {symbol}."""
        pass
"#,
            symbol = request.symbol
        );

        CodeGenResult::new(code, request.file.clone())
    }

    /// Generate property accessors.
    fn generate_properties(&self, request: &CodeGenRequest, fields: &[String]) -> CodeGenResult {
        let mut code = String::new();

        for field in fields {
            code.push_str(&format!(
                r#"    @property
    def {field}(self) -> Any:
        """Get {field}."""
        return self._{field}

    @{field}.setter
    def {field}(self, value: Any) -> None:
        """Set {field}."""
        self._{field} = value

"#,
                field = field
            ));
        }

        CodeGenResult::new(code, request.file.clone()).with_import("from typing import Any")
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docstring_generation() {
        let gen = CodeGenerator::new();
        let request = CodeGenRequest {
            kind: CodeGenKind::Docstring {
                style: DocstringStyle::Google,
            },
            file: PathBuf::from("test.py"),
            symbol: "my_function".to_string(),
            options: CodeGenOptions::default(),
        };

        let result = gen.generate(&request);
        assert!(result.code.contains("my_function"));
        assert!(result.code.contains("Args:"));
    }

    #[test]
    fn test_pytest_generation() {
        let gen = CodeGenerator::new();
        let request = CodeGenRequest {
            kind: CodeGenKind::TestStub {
                framework: TestFramework::Pytest,
            },
            file: PathBuf::from("test.py"),
            symbol: "MyClass".to_string(),
            options: CodeGenOptions::default(),
        };

        let result = gen.generate(&request);
        assert!(result.code.contains("class TestMyClass"));
        assert!(result.imports.contains(&"import pytest".to_string()));
    }

    #[test]
    fn test_type_stub_generation() {
        let gen = CodeGenerator::new();
        let request = CodeGenRequest {
            kind: CodeGenKind::TypeStub,
            file: PathBuf::from("module.py"),
            symbol: "my_func".to_string(),
            options: CodeGenOptions::default(),
        };

        let result = gen.generate(&request);
        assert!(result.target_file.extension().unwrap() == "pyi");
    }

    #[test]
    fn test_module_stub_generation() {
        use crate::type_inference::{Param, ParamKind, Type, TypeBinding, TypeContext};

        // Create a CodeGenerator with a pre-populated TypeContext
        let mut type_context = TypeContext::new();
        let file = PathBuf::from("mymodule.py");

        // Add a function binding
        let binding = TypeBinding {
            ty: Type::Callable {
                params: vec![Param {
                    name: "x".to_string(),
                    ty: Type::Int,
                    has_default: false,
                    kind: ParamKind::Positional,
                }],
                ret: Box::new(Type::Str),
            },
            source_file: file.clone(),
            symbol: "my_function".to_string(),
            line: 1,
            is_exported: true,
            dependencies: vec![],
            is_propagated: false,
        };
        type_context.add_binding(file.clone(), binding);

        let gen = CodeGenerator {
            type_context,
            default_options: CodeGenOptions::default(),
        };

        let request = CodeGenRequest {
            kind: CodeGenKind::ModuleStub,
            file: file.clone(),
            symbol: "my_function".to_string(),
            options: CodeGenOptions::default(),
        };

        let result = gen.generate(&request);

        // Verify .pyi extension
        assert_eq!(result.target_file.extension().unwrap(), "pyi");
        assert_eq!(result.target_file.file_stem().unwrap(), "mymodule");

        // Verify stub contains imports
        assert!(result.code.contains("from typing import Any"));

        // Verify stub contains __all__ list
        assert!(result.code.contains("__all__ = [\"my_function\"]"));

        // Verify function stub
        assert!(result.code.contains("def my_function(x: int) -> str: ..."));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/type_inference/codegen.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/type_inference/codegen.rs` captured during libs codegen standardization.
```

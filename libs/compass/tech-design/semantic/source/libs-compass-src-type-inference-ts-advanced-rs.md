---
id: libs-compass-src-type-inference-ts-advanced-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/type_inference/ts_advanced.rs`.
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

# Standardized libs/compass/src/type_inference/ts_advanced.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/type_inference/ts_advanced.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `TsGenericApplication` | libs/compass/src/type_inference/ts_advanced.rs | struct | pub | 29 | pub struct TsGenericApplication { |
| `ConstraintResult` | libs/compass/src/type_inference/ts_advanced.rs | enum | pub | 43 | pub enum ConstraintResult { |
| `AdvancedTsTypeInferencer` | libs/compass/src/type_inference/ts_advanced.rs | struct | pub | 62 | pub struct AdvancedTsTypeInferencer<'ctx> { |
| `new` | libs/compass/src/type_inference/ts_advanced.rs | function | pub | 68 | pub fn new(ctx: &'ctx TsTypeContext) -> Self { |
| `evaluate_mapped_type` | libs/compass/src/type_inference/ts_advanced.rs | function | pub | 81 | pub fn evaluate_mapped_type( |
| `keyof` | libs/compass/src/type_inference/ts_advanced.rs | function | pub | 118 | pub fn keyof(&self, interface_name: &str) -> Vec<String> { |
| `evaluate_conditional_type` | libs/compass/src/type_inference/ts_advanced.rs | function | pub | 138 | pub fn evaluate_conditional_type( |
| `check_extends` | libs/compass/src/type_inference/ts_advanced.rs | function | pub | 150 | pub fn check_extends(&self, check_type: &Type, extends_type: &Type) -> ConstraintResult { |
| `matches_template_literal` | libs/compass/src/type_inference/ts_advanced.rs | function | pub | 175 | pub fn matches_template_literal(&self, template: &TsTemplateLiteralType, value: &str) -> bool { |
| `evaluate_template_literal` | libs/compass/src/type_inference/ts_advanced.rs | function | pub | 212 | pub fn evaluate_template_literal( |
| `infer_generic_call` | libs/compass/src/type_inference/ts_advanced.rs | function | pub | 235 | pub fn infer_generic_call( |
| `apply_substitutions` | libs/compass/src/type_inference/ts_advanced.rs | function | pub | 339 | pub fn apply_substitutions(&self, ty: &Type, subs: &HashMap<TypeVarId, Type>) -> Type { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Advanced TypeScript type inference (R1)
//!
//! Extends the existing TypeScript type system (`ts_types`, `ts_infer`) with
//! handlers for the advanced constructs most common in modern TypeScript:
//!
//! - **Generics** — constraint checking and type-argument substitution
//! - **Mapped types** — `{ [K in keyof T]: U }` evaluation
//! - **Conditional types** — `T extends U ? X : Y` evaluation
//! - **Template literal types** — `` `${string}-${number}` `` matching
//!
//! The `AdvancedTsTypeInferencer` is used by `hover` and `type-at` handlers
//! to produce accurate type information when the cursor sits on a complex
//! expression whose type cannot be resolved by the base inferencer alone.

use std::collections::HashMap;

use super::ts_types::{
    is_assignable_to, TemplatePart, TsConditionalType, TsMappedType, TsTemplateLiteralType,
    TsTypeContext, TsTypeParam,
};
use super::ty::{Type, TypeVarId};

// ============================================================================
// Generic type application
// ============================================================================

/// A concrete generic type application: `Promise<string>`, `Array<T>`.
#[derive(Debug, Clone, PartialEq)]
pub struct TsGenericApplication {
    /// Name of the generic type (e.g. `"Promise"`, `"ReadonlyArray"`)
    pub name: String,
    /// Supplied type arguments
    pub args: Vec<Type>,
}

// ============================================================================
// Constraint checking
// ============================================================================

/// Result of checking whether a concrete type satisfies a type-parameter
/// constraint (`T extends Constraint`).
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintResult {
    /// The type satisfies the constraint.
    Satisfied,
    /// The type violates the constraint.
    Violated { reason: String },
    /// Satisfiability could not be determined (e.g. unresolved type variable).
    Unknown,
}

// ============================================================================
// AdvancedTsTypeInferencer
// ============================================================================

/// Resolves advanced TypeScript type constructs against a shared
/// `TsTypeContext`.
///
/// Create with [`AdvancedTsTypeInferencer::new`], passing a reference to the
/// `TsTypeContext` populated by the base `TsTypeInferencer`, then call the
/// resolution methods as needed.
pub struct AdvancedTsTypeInferencer<'ctx> {
    ctx: &'ctx TsTypeContext,
}

impl<'ctx> AdvancedTsTypeInferencer<'ctx> {
    /// Create a new inferencer backed by an existing `TsTypeContext`.
    pub fn new(ctx: &'ctx TsTypeContext) -> Self {
        Self { ctx }
    }

    // -----------------------------------------------------------------------
    // Mapped types
    // -----------------------------------------------------------------------

    /// Evaluate a mapped type against a concrete interface name and return the
    /// resulting `property_name → Type` map.
    ///
    /// Handles the common `Partial<T>`, `Required<T>`, and `Readonly<T>`
    /// patterns where the key set comes from `keyof T`.
    pub fn evaluate_mapped_type(
        &self,
        mapped: &TsMappedType,
        interface_name: &str,
    ) -> HashMap<String, Type> {
        let mut result = HashMap::new();

        // Collect the target keys from the interface.
        let keys = self.keyof(interface_name);
        if keys.is_empty() {
            return result;
        }

        for key in &keys {
            // Produce the value type for this key.
            //
            // For simple cases the value type is the property type from the
            // source interface; for more complex mappings it may be the
            // mapped `value_type` directly.
            let value_ty = if let Some(iface) = self.ctx.interfaces.get(interface_name) {
                iface
                    .properties
                    .get(key.as_str())
                    .cloned()
                    .or_else(|| iface.optional_properties.get(key.as_str()).cloned())
                    .unwrap_or_else(|| mapped.value_type.clone())
            } else {
                mapped.value_type.clone()
            };

            result.insert(key.clone(), value_ty);
        }

        result
    }

    /// Return the property names of a named interface (`keyof T`).
    pub fn keyof(&self, interface_name: &str) -> Vec<String> {
        if let Some(iface) = self.ctx.interfaces.get(interface_name) {
            let mut keys: Vec<String> = iface.properties.keys().cloned().collect();
            keys.extend(iface.optional_properties.keys().cloned());
            keys.sort();
            keys
        } else {
            vec![]
        }
    }

    // -----------------------------------------------------------------------
    // Conditional types
    // -----------------------------------------------------------------------

    /// Evaluate a conditional type `T extends U ? X : Y` given a concrete
    /// substitution for `T`.
    ///
    /// Returns the true branch when `T` is assignable to `U`, the false branch
    /// otherwise, and the `Unknown` type when assignability cannot be decided.
    pub fn evaluate_conditional_type(
        &self,
        cond: &TsConditionalType,
        subs: &HashMap<TypeVarId, Type>,
    ) -> Type {
        // Evaluate using the existing `TsConditionalType::evaluate` method
        // which already applies substitutions and checks assignability.
        cond.evaluate(subs)
    }

    /// Check whether `check_type` extends (is assignable to) `extends_type`
    /// in the context of the current type context.
    pub fn check_extends(&self, check_type: &Type, extends_type: &Type) -> ConstraintResult {
        if is_assignable_to(check_type, extends_type) {
            ConstraintResult::Satisfied
        } else {
            match (check_type, extends_type) {
                (Type::TypeVar { .. }, _) | (_, Type::TypeVar { .. }) => ConstraintResult::Unknown,
                _ => ConstraintResult::Violated {
                    reason: format!(
                        "type '{:?}' is not assignable to constraint '{:?}'",
                        check_type, extends_type
                    ),
                },
            }
        }
    }

    // -----------------------------------------------------------------------
    // Template literal types
    // -----------------------------------------------------------------------

    /// Test whether a concrete string `value` matches the pattern described by
    /// a `TsTemplateLiteralType`.
    ///
    /// Only handles the common cases where spans are literal strings or simple
    /// primitive type placeholders (`string`, `number`, `boolean`).
    pub fn matches_template_literal(&self, template: &TsTemplateLiteralType, value: &str) -> bool {
        // Build a regex pattern from the template parts.
        let mut pattern = String::from("^");
        for part in &template.parts {
            match part {
                TemplatePart::Literal(s) => {
                    // Escape regex meta-characters in literal segments.
                    for ch in s.chars() {
                        if "^$.*+?()[]{}|\\".contains(ch) {
                            pattern.push('\\');
                        }
                        pattern.push(ch);
                    }
                }
                TemplatePart::Placeholder(ty) => {
                    let seg = match ty {
                        Type::Int => r"\d+",
                        Type::Float => r"\d+(?:\.\d+)?",
                        Type::Bool => r"(?:true|false)",
                        // string / any / unknown — match one or more characters
                        _ => r".+",
                    };
                    pattern.push_str(seg);
                }
            }
        }
        pattern.push('$');

        if let Ok(re) = regex_lite::Regex::new(&pattern) {
            re.is_match(value)
        } else {
            false
        }
    }

    /// Evaluate a template literal type with concrete type substitutions and
    /// return the resulting `Type`.
    pub fn evaluate_template_literal(
        &self,
        template: &TsTemplateLiteralType,
        subs: &HashMap<TypeVarId, Type>,
    ) -> Type {
        template.evaluate(subs)
    }

    // -----------------------------------------------------------------------
    // Generic call inference
    // -----------------------------------------------------------------------

    /// Infer type arguments for a generic function call.
    ///
    /// Given:
    /// - `type_params` — the function's declared type parameters (with optional
    ///   constraints)
    /// - `param_types` — the function parameter types (may reference type
    ///   variable names)
    /// - `arg_types` — the concrete argument types provided at the call site
    ///
    /// Returns a `TypeVarId → Type` substitution map on success, or an error
    /// message when a constraint is violated or there is an arity mismatch.
    pub fn infer_generic_call(
        &self,
        type_params: &[TsTypeParam],
        param_types: &[Type],
        arg_types: &[Type],
    ) -> Result<HashMap<TypeVarId, Type>, String> {
        if param_types.len() != arg_types.len() {
            return Err(format!(
                "arity mismatch: expected {} argument(s), got {}",
                param_types.len(),
                arg_types.len()
            ));
        }

        // Step 1 — infer type variable bindings from argument positions.
        let mut bindings: HashMap<String, Type> = HashMap::new();
        for (param_ty, arg_ty) in param_types.iter().zip(arg_types.iter()) {
            self.unify_type_var_names(param_ty, arg_ty, type_params, &mut bindings);
        }

        // Step 2 — check constraints for every inferred type variable.
        for (idx, tp) in type_params.iter().enumerate() {
            if let Some(constraint) = &tp.constraint {
                if let Some(inferred_ty) = bindings.get(&tp.name) {
                    if !is_assignable_to(inferred_ty, constraint) {
                        return Err(format!(
                            "type '{:?}' does not satisfy constraint '{:?}' for parameter '{}'",
                            inferred_ty, constraint, tp.name
                        ));
                    }
                } else if tp.default.is_none() {
                    // No binding and no default: use the constraint as the type.
                    bindings.insert(tp.name.clone(), constraint.clone());
                }

                let _ = idx; // suppress unused warning
            }
        }

        // Step 3 — build the final TypeVarId-keyed substitution map.
        let mut subs: HashMap<TypeVarId, Type> = HashMap::new();
        for (idx, tp) in type_params.iter().enumerate() {
            let id = TypeVarId(idx);
            if let Some(ty) = bindings.remove(&tp.name) {
                subs.insert(id, ty);
            } else if let Some(default) = &tp.default {
                subs.insert(id, default.clone());
            }
        }

        Ok(subs)
    }

    /// Walk `param_ty` and `arg_ty` together, recording name → type bindings
    /// for any type parameter name found in `param_ty`.
    fn unify_type_var_names(
        &self,
        param_ty: &Type,
        arg_ty: &Type,
        type_params: &[TsTypeParam],
        bindings: &mut HashMap<String, Type>,
    ) {
        let _param_names: Vec<&str> = type_params.iter().map(|tp| tp.name.as_str()).collect();

        match param_ty {
            Type::TypeVar { name, .. } => {
                // Named type variable — record the binding.
                bindings
                    .entry(name.clone())
                    .or_insert_with(|| arg_ty.clone());
            }
            Type::Instance {
                name: inst_name,
                type_args: inst_args,
                ..
            } => {
                // Recurse into generic type arguments.
                if let Type::Instance {
                    name: arg_name,
                    type_args: arg_args,
                    ..
                } = arg_ty
                {
                    if inst_name == arg_name {
                        for (pa, aa) in inst_args.iter().zip(arg_args.iter()) {
                            self.unify_type_var_names(pa, aa, type_params, bindings);
                        }
                    }
                }
            }
            Type::List(inner_p) => {
                if let Type::List(inner_a) = arg_ty {
                    self.unify_type_var_names(inner_p, inner_a, type_params, bindings);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Substitution helpers
    // -----------------------------------------------------------------------

    /// Apply a `TypeVarId → Type` substitution map to a `Type`.
    pub fn apply_substitutions(&self, ty: &Type, subs: &HashMap<TypeVarId, Type>) -> Type {
        ty.substitute(subs)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::super::ts_types::TsInterface;
    use super::*;

    fn empty_ctx() -> TsTypeContext {
        TsTypeContext::new()
    }

    // -----------------------------------------------------------------------
    // Template literal matching
    // -----------------------------------------------------------------------

    #[test]
    fn test_template_match_string_interpolation() {
        let ctx = empty_ctx();
        let inf = AdvancedTsTypeInferencer::new(&ctx);

        let template = TsTemplateLiteralType::new("event-", Type::Str, "");
        assert!(inf.matches_template_literal(&template, "event-click"));
        assert!(inf.matches_template_literal(&template, "event-mousedown"));
        assert!(!inf.matches_template_literal(&template, "noevent"));
    }

    #[test]
    fn test_template_match_number_interpolation() {
        let ctx = empty_ctx();
        let inf = AdvancedTsTypeInferencer::new(&ctx);

        let template = TsTemplateLiteralType::new("col-", Type::Int, "");
        assert!(inf.matches_template_literal(&template, "col-12"));
        assert!(!inf.matches_template_literal(&template, "col-abc"));
    }

    #[test]
    fn test_template_no_interpolation() {
        let ctx = empty_ctx();
        let inf = AdvancedTsTypeInferencer::new(&ctx);

        let template = TsTemplateLiteralType {
            parts: vec![TemplatePart::Literal("hello".to_string())],
        };
        assert!(inf.matches_template_literal(&template, "hello"));
        assert!(!inf.matches_template_literal(&template, "world"));
    }

    // -----------------------------------------------------------------------
    // Conditional type evaluation
    // -----------------------------------------------------------------------

    #[test]
    fn test_conditional_true_branch() {
        let ctx = empty_ctx();
        let inf = AdvancedTsTypeInferencer::new(&ctx);

        let cond = TsConditionalType {
            check_type: Box::new(Type::Str),
            extends_type: Box::new(Type::Str),
            true_type: Box::new(Type::Bool),
            false_type: Box::new(Type::Int),
        };
        let result = inf.evaluate_conditional_type(&cond, &HashMap::new());
        assert_eq!(result, Type::Bool);
    }

    #[test]
    fn test_conditional_false_branch() {
        let ctx = empty_ctx();
        let inf = AdvancedTsTypeInferencer::new(&ctx);

        let cond = TsConditionalType {
            check_type: Box::new(Type::Int),
            extends_type: Box::new(Type::Str),
            true_type: Box::new(Type::Bool),
            false_type: Box::new(Type::Float),
        };
        let result = inf.evaluate_conditional_type(&cond, &HashMap::new());
        assert_eq!(result, Type::Float);
    }

    // -----------------------------------------------------------------------
    // Mapped type evaluation
    // -----------------------------------------------------------------------

    #[test]
    fn test_mapped_type_keyof_interface() {
        let mut ctx = TsTypeContext::new();
        let mut iface = TsInterface::new("User".to_string());
        iface.properties.insert("name".to_string(), Type::Str);
        iface.properties.insert("age".to_string(), Type::Int);
        ctx.register_interface(iface);

        let inf = AdvancedTsTypeInferencer::new(&ctx);

        let mapped = TsMappedType {
            key_var: "K".to_string(),
            keys: Type::Str,
            value_type: Type::Optional(Box::new(Type::Str)),
            optional_modifier: None,
            readonly_modifier: None,
        };

        let result = inf.evaluate_mapped_type(&mapped, "User");
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("name"));
        assert!(result.contains_key("age"));
    }

    #[test]
    fn test_keyof_unknown_interface() {
        let ctx = empty_ctx();
        let inf = AdvancedTsTypeInferencer::new(&ctx);
        assert!(inf.keyof("NonExistent").is_empty());
    }

    // -----------------------------------------------------------------------
    // Generic call inference
    // -----------------------------------------------------------------------

    #[test]
    fn test_infer_identity_function() {
        let ctx = empty_ctx();
        let inf = AdvancedTsTypeInferencer::new(&ctx);

        let type_params = vec![TsTypeParam::new("T".to_string())];
        // function identity<T>(x: T): T  →  identity("hello")
        let param_types = vec![Type::TypeVar {
            id: TypeVarId(0),
            name: "T".to_string(),
            bound: None,
            constraints: vec![],
            variance: super::super::ty::Variance::Invariant,
        }];
        let arg_types = vec![Type::Str];

        let subs = inf
            .infer_generic_call(&type_params, &param_types, &arg_types)
            .expect("inference should succeed");
        assert_eq!(subs.get(&TypeVarId(0)), Some(&Type::Str));
    }

    #[test]
    fn test_infer_arity_mismatch() {
        let ctx = empty_ctx();
        let inf = AdvancedTsTypeInferencer::new(&ctx);
        let type_params = vec![TsTypeParam::new("T".to_string())];
        let result = inf.infer_generic_call(&type_params, &[Type::Str], &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_infer_constraint_violation() {
        let ctx = empty_ctx();
        let inf = AdvancedTsTypeInferencer::new(&ctx);

        // function add<T extends number>(x: T): T  →  add("text")  ← error
        let type_params = vec![TsTypeParam::new("T".to_string()).with_constraint(Type::Float)];
        let param_types = vec![Type::TypeVar {
            id: TypeVarId(0),
            name: "T".to_string(),
            bound: Some(Box::new(Type::Float)),
            constraints: vec![],
            variance: super::super::ty::Variance::Invariant,
        }];
        let arg_types = vec![Type::Str];

        let result = inf.infer_generic_call(&type_params, &param_types, &arg_types);
        assert!(result.is_err(), "should fail constraint check");
    }

    // -----------------------------------------------------------------------
    // check_extends
    // -----------------------------------------------------------------------

    #[test]
    fn test_check_extends_satisfied() {
        let ctx = empty_ctx();
        let inf = AdvancedTsTypeInferencer::new(&ctx);
        assert_eq!(
            inf.check_extends(&Type::Int, &Type::Float),
            ConstraintResult::Satisfied
        );
    }

    #[test]
    fn test_check_extends_violated() {
        let ctx = empty_ctx();
        let inf = AdvancedTsTypeInferencer::new(&ctx);
        let result = inf.check_extends(&Type::Str, &Type::Int);
        assert!(matches!(result, ConstraintResult::Violated { .. }));
    }

    #[test]
    fn test_check_extends_unknown_for_typevar() {
        let ctx = empty_ctx();
        let inf = AdvancedTsTypeInferencer::new(&ctx);
        let tv = Type::TypeVar {
            id: TypeVarId(0),
            name: "T".to_string(),
            bound: None,
            constraints: vec![],
            variance: super::super::ty::Variance::Invariant,
        };
        assert_eq!(
            inf.check_extends(&tv, &Type::Str),
            ConstraintResult::Unknown
        );
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/type_inference/ts_advanced.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/type_inference/ts_advanced.rs` captured during libs codegen standardization.
```

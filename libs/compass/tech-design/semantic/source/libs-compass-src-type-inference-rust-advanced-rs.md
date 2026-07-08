---
id: libs-compass-src-type-inference-rust-advanced-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/type_inference/rust_advanced.rs`.
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

# Standardized libs/compass/src/type_inference/rust_advanced.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/type_inference/rust_advanced.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ArraySizeExpr` | libs/compass/src/type_inference/rust_advanced.rs | enum | pub | 29 | pub enum ArraySizeExpr { |
| `SizeOp` | libs/compass/src/type_inference/rust_advanced.rs | enum | pub | 44 | pub enum SizeOp { |
| `evaluate` | libs/compass/src/type_inference/rust_advanced.rs | function | pub | 57 | pub fn evaluate(&self, env: &HashMap<String, usize>) -> Option<usize> { |
| `into_array_type` | libs/compass/src/type_inference/rust_advanced.rs | function | pub | 82 | pub fn into_array_type(&self, element: RustType, env: &HashMap<String, usize>) -> RustType { |
| `AssocTypeConstraint` | libs/compass/src/type_inference/rust_advanced.rs | struct | pub | 99 | pub struct AssocTypeConstraint { |
| `ComplexTraitBounds` | libs/compass/src/type_inference/rust_advanced.rs | struct | pub | 112 | pub struct ComplexTraitBounds { |
| `BoundCheckResult` | libs/compass/src/type_inference/rust_advanced.rs | enum | pub | 125 | pub enum BoundCheckResult { |
| `check_complex_trait_bounds` | libs/compass/src/type_inference/rust_advanced.rs | function | pub | 147 | pub fn check_complex_trait_bounds( |
| `ProjectionResolver` | libs/compass/src/type_inference/rust_advanced.rs | struct | pub | 256 | pub struct ProjectionResolver<'ctx> { |
| `new` | libs/compass/src/type_inference/rust_advanced.rs | function | pub | 262 | pub fn new(ctx: &'ctx RustTypeContext) -> Self { |
| `resolve_projection` | libs/compass/src/type_inference/rust_advanced.rs | function | pub | 273 | pub fn resolve_projection( |
| `build_projection` | libs/compass/src/type_inference/rust_advanced.rs | function | pub | 317 | pub fn build_projection( |
| `ElisionRule` | libs/compass/src/type_inference/rust_advanced.rs | enum | pub | 346 | pub enum ElisionRule { |
| `ElisionResult` | libs/compass/src/type_inference/rust_advanced.rs | struct | pub | 359 | pub struct ElisionResult { |
| `apply_lifetime_elision` | libs/compass/src/type_inference/rust_advanced.rs | function | pub | 377 | pub fn apply_lifetime_elision( |
| `RustAdvancedInferencer` | libs/compass/src/type_inference/rust_advanced.rs | struct | pub | 436 | pub struct RustAdvancedInferencer<'ctx> { |
| `check_bounds` | libs/compass/src/type_inference/rust_advanced.rs | function | pub | 464 | pub fn check_bounds( |
| `evaluate_array_size` | libs/compass/src/type_inference/rust_advanced.rs | function | pub | 473 | pub fn evaluate_array_size( |
| `apply_elision` | libs/compass/src/type_inference/rust_advanced.rs | function | pub | 482 | pub fn apply_elision( |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Advanced Rust type inference (R2)
//!
//! Extends the existing Rust type system (`rust_types`, `rust_infer`) with
//! handlers for the complex Rust-specific constructs that the base inferencer
//! does not cover:
//!
//! - **Array size expressions** — const-generic expressions `[T; N*2]`
//! - **Complex trait bounds** — multi-bound where clauses with associated-type
//!   equality constraints (`T: Iterator<Item = u32> + Send + 'static`)
//! - **Associated type projections** — `<T as Trait>::Assoc` resolution
//! - **Lifetime elision** — all three standard elision rules for `fn` signatures
//!   and `impl` blocks

use std::collections::HashMap;

use super::rust_infer::RustTypeContext;
use super::rust_types::{Lifetime, LifetimeId, RustType, TraitBound, TraitId, TraitRef};

// ============================================================================
// R2a: Array size expressions
// ============================================================================

/// A constant expression that may appear as an array size.
///
/// Rust supports full const-generic expressions: `[T; N]`, `[T; N + M]`,
/// `[T; size_of::<u64>()]`, etc.  This type models the subset that Lens
/// needs to evaluate for `hover` / `type-at` purposes.
#[derive(Debug, Clone, PartialEq)]
pub enum ArraySizeExpr {
    /// A literal integer size
    Literal(usize),
    /// A named const-generic parameter
    ConstParam(String),
    /// Binary arithmetic expression
    BinOp {
        op: SizeOp,
        lhs: Box<ArraySizeExpr>,
        rhs: Box<ArraySizeExpr>,
    },
}

/// Binary operator for array size expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl ArraySizeExpr {
    /// Attempt to evaluate the expression given a mapping of const-param names
    /// to their concrete `usize` values.
    ///
    /// Returns `None` when a const parameter is not in `env` or on arithmetic
    /// overflow / division-by-zero.
    pub fn evaluate(&self, env: &HashMap<String, usize>) -> Option<usize> {
        match self {
            ArraySizeExpr::Literal(n) => Some(*n),
            ArraySizeExpr::ConstParam(name) => env.get(name).copied(),
            ArraySizeExpr::BinOp { op, lhs, rhs } => {
                let l = lhs.evaluate(env)?;
                let r = rhs.evaluate(env)?;
                match op {
                    SizeOp::Add => l.checked_add(r),
                    SizeOp::Sub => l.checked_sub(r),
                    SizeOp::Mul => l.checked_mul(r),
                    SizeOp::Div => {
                        if r == 0 {
                            None
                        } else {
                            Some(l / r)
                        }
                    }
                }
            }
        }
    }

    /// Resolve the `ArraySizeExpr` to a concrete `RustType::Array` variant
    /// given a const-param environment.
    pub fn into_array_type(&self, element: RustType, env: &HashMap<String, usize>) -> RustType {
        let size = self.evaluate(env).unwrap_or(0);
        RustType::Array {
            element: Box::new(element),
            size,
        }
    }
}

// ============================================================================
// R2b: Complex trait bounds
// ============================================================================

/// An associated-type constraint within a trait bound.
///
/// Corresponds to `<Item = u32>` in `T: Iterator<Item = u32>`.
#[derive(Debug, Clone, PartialEq)]
pub struct AssocTypeConstraint {
    /// Name of the associated type (e.g. `"Item"`)
    pub name: String,
    /// Required concrete type
    pub ty: RustType,
}

/// A complex trait bound for a single type variable, combining multiple
/// `TraitBound`s and associated-type equality constraints.
///
/// Corresponds to a `where` clause predicate like:
/// `T: Iterator<Item = u32> + DoubleEndedIterator + Send + 'static`
#[derive(Debug, Clone)]
pub struct ComplexTraitBounds {
    /// The type being bounded (usually a `TypeParam`)
    pub ty: RustType,
    /// Required trait bounds
    pub trait_bounds: Vec<TraitBound>,
    /// Associated-type equality constraints
    pub assoc_constraints: Vec<AssocTypeConstraint>,
    /// Required lifetime bounds
    pub lifetime_bounds: Vec<Lifetime>,
}

/// Result of checking a type against a `ComplexTraitBounds` spec.
#[derive(Debug, Clone, PartialEq)]
pub enum BoundCheckResult {
    /// All bounds are satisfied.
    Satisfied,
    /// One or more trait bounds are not satisfied.
    MissingTraits(Vec<String>),
    /// One or more associated-type constraints are violated.
    AssocTypeMismatch {
        trait_name: String,
        assoc_name: String,
        expected: RustType,
        got: Option<RustType>,
    },
    /// Insufficient information — result is unknown.
    Unknown,
}

/// Check whether the impl blocks in `ctx` satisfy all the bounds in `spec`
/// for the concrete type `concrete_type`.
///
/// This performs a structural walk: for each required trait, it looks up
/// impl blocks in the context, verifies the impl covers the concrete type,
/// and then checks associated-type constraints.
pub fn check_complex_trait_bounds(
    ctx: &RustTypeContext,
    concrete_type: &RustType,
    spec: &ComplexTraitBounds,
) -> BoundCheckResult {
    let mut missing_traits: Vec<String> = Vec::new();

    for bound in &spec.trait_bounds {
        if bound.is_negative {
            // Negative bounds (`!Send`) are assumed satisfied unless we can
            // prove otherwise — conservative approximation.
            continue;
        }

        let trait_name = &bound.trait_ref.name;
        let impl_found = ctx.trait_impls.iter().any(|imp| {
            imp.trait_ref
                .as_ref()
                .map_or(false, |tr| &tr.name == trait_name)
                && types_match(&imp.self_type, concrete_type)
        });

        if !impl_found {
            // Check if the trait_def is a built-in auto trait (Send/Sync/Sized).
            let is_auto = ctx
                .trait_defs
                .values()
                .any(|def| def.name == *trait_name && def.is_auto);

            if !is_auto {
                missing_traits.push(trait_name.clone());
            }
        }
    }

    if !missing_traits.is_empty() {
        return BoundCheckResult::MissingTraits(missing_traits);
    }

    // Check associated-type constraints.
    for constraint in &spec.assoc_constraints {
        // Find an impl that provides this associated type.
        let assoc_ty = ctx.trait_impls.iter().find_map(|imp| {
            if types_match(&imp.self_type, concrete_type) {
                imp.associated_types
                    .iter()
                    .find(|(n, _)| n == &constraint.name)
                    .map(|(_, t)| t.clone())
            } else {
                None
            }
        });

        match assoc_ty {
            Some(ref got) if got == &constraint.ty => {}
            Some(got) => {
                // We need to find the trait name for this associated type.
                let trait_name = spec
                    .trait_bounds
                    .first()
                    .map(|b| b.trait_ref.name.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                return BoundCheckResult::AssocTypeMismatch {
                    trait_name,
                    assoc_name: constraint.name.clone(),
                    expected: constraint.ty.clone(),
                    got: Some(got),
                };
            }
            None => {
                // Associated type not found in any impl; if the trait provides
                // a default we consider it satisfied.
            }
        }
    }

    BoundCheckResult::Satisfied
}

/// Structural type equality check (ignores lifetime arguments for simplicity).
fn types_match(a: &RustType, b: &RustType) -> bool {
    match (a, b) {
        (
            RustType::Named {
                name: n1,
                type_args: ta1,
                ..
            },
            RustType::Named {
                name: n2,
                type_args: ta2,
                ..
            },
        ) => {
            n1 == n2
                && ta1.len() == ta2.len()
                && ta1.iter().zip(ta2.iter()).all(|(x, y)| types_match(x, y))
        }
        (RustType::TypeParam { name: n1, .. }, RustType::TypeParam { name: n2, .. }) => n1 == n2,
        _ => std::mem::discriminant(a) == std::mem::discriminant(b),
    }
}

// ============================================================================
// R2c: Associated type projections
// ============================================================================

/// Resolution context for `<T as Trait>::Assoc` projections.
pub struct ProjectionResolver<'ctx> {
    ctx: &'ctx RustTypeContext,
}

impl<'ctx> ProjectionResolver<'ctx> {
    /// Create a resolver backed by an existing `RustTypeContext`.
    pub fn new(ctx: &'ctx RustTypeContext) -> Self {
        Self { ctx }
    }

    /// Resolve an associated type projection `<concrete_type as trait_name>::assoc_name`.
    ///
    /// Looks through all impl blocks in the context for one that:
    /// 1. implements `trait_name` for `concrete_type`, and
    /// 2. provides a concrete value for `assoc_name`.
    ///
    /// Returns `None` when no matching impl is found.
    pub fn resolve_projection(
        &self,
        concrete_type: &RustType,
        trait_name: &str,
        assoc_name: &str,
    ) -> Option<RustType> {
        for imp in self.ctx.trait_impls.iter() {
            let impl_trait_name = imp
                .trait_ref
                .as_ref()
                .map(|tr| tr.name.as_str())
                .unwrap_or("");

            if impl_trait_name != trait_name {
                continue;
            }
            if !types_match(&imp.self_type, concrete_type) {
                continue;
            }

            if let Some((_, ty)) = imp.associated_types.iter().find(|(n, _)| n == assoc_name) {
                return Some(ty.clone());
            }
        }

        // Fall back: look up the associated type's default in the trait definition.
        for trait_def in self.ctx.trait_defs.values() {
            if trait_def.name != trait_name {
                continue;
            }
            if let Some(at) = trait_def
                .associated_types
                .iter()
                .find(|at| at.name == assoc_name)
            {
                return at.default.clone();
            }
        }

        None
    }

    /// Build a `RustType::Projection` node for a given type, trait, and
    /// associated type name.
    pub fn build_projection(
        &self,
        base_type: RustType,
        trait_id: TraitId,
        trait_name: impl Into<String>,
        assoc_name: impl Into<String>,
    ) -> RustType {
        let trait_ref = TraitRef {
            trait_id,
            name: trait_name.into(),
            type_args: vec![],
            lifetime_args: vec![],
        };
        RustType::Projection {
            base: Box::new(base_type),
            trait_ref: Some(trait_ref),
            name: assoc_name.into(),
        }
    }
}

// ============================================================================
// R2d: Lifetime elision
// ============================================================================

/// Rust lifetime elision rule applied to a function or method signature.
///
/// See <https://doc.rust-lang.org/reference/lifetime-elision.html>.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElisionRule {
    /// Rule 1 — each elided input lifetime gets its own fresh lifetime.
    EachInputGetsOwn,
    /// Rule 2 — if there is exactly one input lifetime, that lifetime is
    ///           assigned to all elided output lifetimes.
    SingleInputToOutput,
    /// Rule 3 — if the function has `&self` or `&mut self`, the lifetime of
    ///           that reference is assigned to all elided output lifetimes.
    SelfToOutput,
}

/// The result of applying lifetime elision to a function signature.
#[derive(Debug, Clone, PartialEq)]
pub struct ElisionResult {
    /// The elision rule that was applied to determine the output lifetime.
    pub rule: ElisionRule,
    /// Assigned output lifetime (if any)
    pub output_lifetime: Option<Lifetime>,
    /// Input lifetimes, each assigned a unique identifier (rule 1)
    pub input_lifetimes: Vec<Lifetime>,
}

/// Apply Rust's three standard lifetime elision rules to a function signature.
///
/// # Parameters
/// - `has_self_ref` — `true` when the first parameter is `&self` or `&mut self`
/// - `input_elided_count` — number of elided (`'_` or anonymous) lifetimes in
///   input position
/// - `has_elided_output` — `true` when the return type contains an elided
///   lifetime
/// - `lifetime_counter` — counter used to generate fresh `LifetimeId`s
pub fn apply_lifetime_elision(
    has_self_ref: bool,
    input_elided_count: usize,
    has_elided_output: bool,
    lifetime_counter: &mut usize,
) -> ElisionResult {
    // Rule 1 — assign a unique lifetime to each elided input lifetime.
    let input_lifetimes: Vec<Lifetime> = (0..input_elided_count)
        .map(|_| {
            let id = LifetimeId(*lifetime_counter);
            *lifetime_counter += 1;
            Lifetime::Inferred(id)
        })
        .collect();

    if !has_elided_output {
        return ElisionResult {
            rule: ElisionRule::EachInputGetsOwn,
            output_lifetime: None,
            input_lifetimes,
        };
    }

    // Rule 3 — self reference takes priority over rule 2.
    if has_self_ref && !input_lifetimes.is_empty() {
        return ElisionResult {
            rule: ElisionRule::SelfToOutput,
            output_lifetime: Some(input_lifetimes[0].clone()),
            input_lifetimes,
        };
    }

    // Rule 2 — exactly one input lifetime ⇒ use it as output lifetime.
    if input_lifetimes.len() == 1 {
        return ElisionResult {
            rule: ElisionRule::SingleInputToOutput,
            output_lifetime: Some(input_lifetimes[0].clone()),
            input_lifetimes,
        };
    }

    // Cannot determine: multiple input lifetimes, no self ref, elided output.
    // This is a compile error in real Rust; return a fresh inferred lifetime
    // as a best-effort fallback.
    let fallback_id = LifetimeId(*lifetime_counter);
    *lifetime_counter += 1;

    ElisionResult {
        rule: ElisionRule::EachInputGetsOwn,
        output_lifetime: Some(Lifetime::Inferred(fallback_id)),
        input_lifetimes,
    }
}

// ============================================================================
// RustAdvancedInferencer — convenience wrapper
// ============================================================================

/// High-level helper that bundles projection resolution and bound checking.
pub struct RustAdvancedInferencer<'ctx> {
    ctx: &'ctx RustTypeContext,
    projection_resolver: ProjectionResolver<'ctx>,
    pub lifetime_counter: usize,
}

impl<'ctx> RustAdvancedInferencer<'ctx> {
    /// Create a new inferencer from an existing `RustTypeContext`.
    pub fn new(ctx: &'ctx RustTypeContext) -> Self {
        Self {
            ctx,
            projection_resolver: ProjectionResolver::new(ctx),
            lifetime_counter: 0,
        }
    }

    /// Resolve an associated type projection.
    pub fn resolve_projection(
        &self,
        concrete_type: &RustType,
        trait_name: &str,
        assoc_name: &str,
    ) -> Option<RustType> {
        self.projection_resolver
            .resolve_projection(concrete_type, trait_name, assoc_name)
    }

    /// Check complex trait bounds for a concrete type.
    pub fn check_bounds(
        &self,
        concrete_type: &RustType,
        spec: &ComplexTraitBounds,
    ) -> BoundCheckResult {
        check_complex_trait_bounds(self.ctx, concrete_type, spec)
    }

    /// Evaluate an array size expression.
    pub fn evaluate_array_size(
        &self,
        expr: &ArraySizeExpr,
        env: &HashMap<String, usize>,
    ) -> Option<usize> {
        expr.evaluate(env)
    }

    /// Apply lifetime elision to a function signature.
    pub fn apply_elision(
        &mut self,
        has_self_ref: bool,
        input_elided_count: usize,
        has_elided_output: bool,
    ) -> ElisionResult {
        apply_lifetime_elision(
            has_self_ref,
            input_elided_count,
            has_elided_output,
            &mut self.lifetime_counter,
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Array size expressions
    // -----------------------------------------------------------------------

    #[test]
    fn test_literal_array_size() {
        let env = HashMap::new();
        let expr = ArraySizeExpr::Literal(16);
        assert_eq!(expr.evaluate(&env), Some(16));
    }

    #[test]
    fn test_const_param_array_size() {
        let mut env = HashMap::new();
        env.insert("N".to_string(), 8_usize);
        let expr = ArraySizeExpr::ConstParam("N".to_string());
        assert_eq!(expr.evaluate(&env), Some(8));
    }

    #[test]
    fn test_binop_add() {
        let mut env = HashMap::new();
        env.insert("N".to_string(), 3_usize);
        env.insert("M".to_string(), 5_usize);
        let expr = ArraySizeExpr::BinOp {
            op: SizeOp::Add,
            lhs: Box::new(ArraySizeExpr::ConstParam("N".to_string())),
            rhs: Box::new(ArraySizeExpr::ConstParam("M".to_string())),
        };
        assert_eq!(expr.evaluate(&env), Some(8));
    }

    #[test]
    fn test_binop_mul() {
        let mut env = HashMap::new();
        env.insert("N".to_string(), 4_usize);
        let expr = ArraySizeExpr::BinOp {
            op: SizeOp::Mul,
            lhs: Box::new(ArraySizeExpr::ConstParam("N".to_string())),
            rhs: Box::new(ArraySizeExpr::Literal(2)),
        };
        assert_eq!(expr.evaluate(&env), Some(8));
    }

    #[test]
    fn test_binop_div_by_zero() {
        let env = HashMap::new();
        let expr = ArraySizeExpr::BinOp {
            op: SizeOp::Div,
            lhs: Box::new(ArraySizeExpr::Literal(10)),
            rhs: Box::new(ArraySizeExpr::Literal(0)),
        };
        assert_eq!(expr.evaluate(&env), None);
    }

    #[test]
    fn test_missing_const_param() {
        let env = HashMap::new();
        let expr = ArraySizeExpr::ConstParam("UNKNOWN".to_string());
        assert_eq!(expr.evaluate(&env), None);
    }

    #[test]
    fn test_into_array_type() {
        let env = HashMap::new();
        let expr = ArraySizeExpr::Literal(4);
        let ty = expr.into_array_type(RustType::U8, &env);
        assert_eq!(
            ty,
            RustType::Array {
                element: Box::new(RustType::U8),
                size: 4,
            }
        );
    }

    // -----------------------------------------------------------------------
    // Lifetime elision
    // -----------------------------------------------------------------------

    #[test]
    fn test_elision_rule1_no_output() {
        let mut counter = 0_usize;
        let result = apply_lifetime_elision(false, 2, false, &mut counter);
        assert_eq!(result.rule, ElisionRule::EachInputGetsOwn);
        assert_eq!(result.input_lifetimes.len(), 2);
        assert!(result.output_lifetime.is_none());
        assert_eq!(counter, 2);
    }

    #[test]
    fn test_elision_rule2_single_input() {
        let mut counter = 0_usize;
        // fn foo<'a>(x: &'a str) -> &str  — rule 2 applies
        let result = apply_lifetime_elision(false, 1, true, &mut counter);
        assert_eq!(result.rule, ElisionRule::SingleInputToOutput);
        assert!(result.output_lifetime.is_some());
        assert_eq!(result.input_lifetimes.len(), 1);
    }

    #[test]
    fn test_elision_rule3_self_ref() {
        let mut counter = 0_usize;
        // fn method(&self, x: &str) -> &str  — rule 3 applies
        let result = apply_lifetime_elision(true, 2, true, &mut counter);
        assert_eq!(result.rule, ElisionRule::SelfToOutput);
        assert!(result.output_lifetime.is_some());
    }

    #[test]
    fn test_elision_no_elided_lifetimes() {
        let mut counter = 0_usize;
        // fn foo(x: i32) -> i32  — no lifetimes at all
        let result = apply_lifetime_elision(false, 0, false, &mut counter);
        assert_eq!(result.input_lifetimes.len(), 0);
        assert!(result.output_lifetime.is_none());
    }

    // -----------------------------------------------------------------------
    // RustAdvancedInferencer integration
    // -----------------------------------------------------------------------

    #[test]
    fn test_advanced_inferencer_array_size() {
        let ctx = RustTypeContext::new();
        let inf = RustAdvancedInferencer::new(&ctx);
        let expr = ArraySizeExpr::Literal(32);
        assert_eq!(inf.evaluate_array_size(&expr, &HashMap::new()), Some(32));
    }

    #[test]
    fn test_advanced_inferencer_elision() {
        let ctx = RustTypeContext::new();
        let mut inf = RustAdvancedInferencer::new(&ctx);
        let result = inf.apply_elision(false, 1, true);
        assert_eq!(result.rule, ElisionRule::SingleInputToOutput);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/type_inference/rust_advanced.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/type_inference/rust_advanced.rs` captured during libs codegen standardization.
```

//! Rust trait resolution
//!
//! This module provides trait resolution functionality for Rust code,
//! including finding trait implementations and resolving method calls.

use std::collections::HashMap;
use std::sync::Arc;

use super::rust_types::*;

// ============================================================================
// Trait Resolution
// ============================================================================

/// Trait resolver for finding implementations
#[derive(Debug, Clone)]
pub struct TraitResolver {
    /// All known trait implementations
    impls: Vec<Arc<ImplBlock>>,
    /// Trait definitions by ID
    trait_defs: HashMap<TraitId, Arc<TraitDef>>,
}

impl TraitResolver {
    /// Create a new trait resolver
    pub fn new() -> Self {
        Self {
            impls: Vec::new(),
            trait_defs: HashMap::new(),
        }
    }

    /// Register a trait definition
    pub fn register_trait(&mut self, trait_def: TraitDef) {
        let id = trait_def.id;
        self.trait_defs.insert(id, Arc::new(trait_def));
    }

    /// Register an impl block
    pub fn register_impl(&mut self, impl_block: ImplBlock) {
        self.impls.push(Arc::new(impl_block));
    }

    /// Find all impls for a given type
    pub fn find_impls_for_type(&self, ty: &RustType) -> Vec<Arc<ImplBlock>> {
        self.impls
            .iter()
            .filter(|impl_block| self.type_matches(&impl_block.self_type, ty))
            .cloned()
            .collect()
    }

    /// Find impl for a specific trait on a type
    pub fn find_trait_impl(&self, ty: &RustType, trait_ref: &TraitRef) -> Option<Arc<ImplBlock>> {
        self.impls
            .iter()
            .find(|impl_block| {
                if let Some(ref impl_trait) = impl_block.trait_ref {
                    impl_trait.trait_id == trait_ref.trait_id
                        && self.type_matches(&impl_block.self_type, ty)
                } else {
                    false
                }
            })
            .cloned()
    }

    /// Check if a type implements a trait
    pub fn implements_trait(&self, ty: &RustType, trait_ref: &TraitRef) -> bool {
        // Check for auto traits
        if let Some(trait_def) = self.trait_defs.get(&trait_ref.trait_id) {
            if trait_def.is_auto {
                // Auto traits are implemented automatically unless explicitly opted out
                return !self.has_negative_impl(ty, trait_ref);
            }
        }

        self.find_trait_impl(ty, trait_ref).is_some()
    }

    /// Check if type has a negative impl for trait
    fn has_negative_impl(&self, ty: &RustType, trait_ref: &TraitRef) -> bool {
        self.impls.iter().any(|impl_block| {
            impl_block.is_negative
                && impl_block.trait_ref.as_ref().map(|t| t.trait_id) == Some(trait_ref.trait_id)
                && self.type_matches(&impl_block.self_type, ty)
        })
    }

    /// Check if two types match (for impl lookup)
    fn type_matches(&self, pattern: &RustType, concrete: &RustType) -> bool {
        match (pattern, concrete) {
            // Exact match
            (a, b) if a == b => true,

            // Type parameter matches anything
            (RustType::TypeParam { .. }, _) => true,

            // Named types must match name and args
            (
                RustType::Named {
                    name: n1,
                    type_args: a1,
                    ..
                },
                RustType::Named {
                    name: n2,
                    type_args: a2,
                    ..
                },
            ) => {
                n1 == n2
                    && a1.len() == a2.len()
                    && a1
                        .iter()
                        .zip(a2.iter())
                        .all(|(p, c)| self.type_matches(p, c))
            }

            // Reference types
            (
                RustType::Reference {
                    mutable: m1,
                    inner: i1,
                    ..
                },
                RustType::Reference {
                    mutable: m2,
                    inner: i2,
                    ..
                },
            ) => m1 == m2 && self.type_matches(i1, i2),

            // Slices
            (RustType::Slice(e1), RustType::Slice(e2)) => self.type_matches(e1, e2),

            // Arrays
            (
                RustType::Array {
                    element: e1,
                    size: s1,
                },
                RustType::Array {
                    element: e2,
                    size: s2,
                },
            ) => s1 == s2 && self.type_matches(e1, e2),

            // Tuples
            (RustType::Tuple(t1), RustType::Tuple(t2)) => {
                t1.len() == t2.len()
                    && t1
                        .iter()
                        .zip(t2.iter())
                        .all(|(p, c)| self.type_matches(p, c))
            }

            _ => false,
        }
    }

    /// Resolve a method call on a type
    pub fn resolve_method(&self, ty: &RustType, method_name: &str) -> Option<MethodResolution> {
        // First check inherent impls
        for impl_block in self.find_impls_for_type(ty) {
            if impl_block.trait_ref.is_none() {
                // Inherent impl
                for method in &impl_block.methods {
                    if method.name == method_name {
                        return Some(MethodResolution {
                            method: method.clone(),
                            impl_block: impl_block.clone(),
                            trait_ref: None,
                        });
                    }
                }
            }
        }

        // Then check trait impls
        for impl_block in self.find_impls_for_type(ty) {
            if let Some(ref trait_ref) = impl_block.trait_ref {
                for method in &impl_block.methods {
                    if method.name == method_name {
                        return Some(MethodResolution {
                            method: method.clone(),
                            impl_block: impl_block.clone(),
                            trait_ref: Some(trait_ref.clone()),
                        });
                    }
                }
            }
        }

        None
    }

    /// Get associated type from trait impl
    pub fn get_associated_type(
        &self,
        ty: &RustType,
        trait_ref: &TraitRef,
        assoc_name: &str,
    ) -> Option<RustType> {
        if let Some(impl_block) = self.find_trait_impl(ty, trait_ref) {
            impl_block
                .associated_types
                .iter()
                .find(|(name, _)| name == assoc_name)
                .map(|(_, ty)| ty.clone())
        } else {
            None
        }
    }
}

impl Default for TraitResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of resolving a method call
#[derive(Debug, Clone)]
pub struct MethodResolution {
    /// The resolved method
    pub method: ImplMethod,
    /// The impl block containing the method
    pub impl_block: Arc<ImplBlock>,
    /// The trait being called (if trait method)
    pub trait_ref: Option<TraitRef>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_resolver_creation() {
        let resolver = TraitResolver::new();
        assert!(resolver.impls.is_empty());
        assert!(resolver.trait_defs.is_empty());
    }

    #[test]
    fn test_register_impl() {
        let mut resolver = TraitResolver::new();
        let impl_block = ImplBlock {
            type_params: vec![],
            trait_ref: None,
            self_type: RustType::I32,
            where_bounds: vec![],
            methods: vec![],
            associated_types: vec![],
            associated_consts: vec![],
            is_negative: false,
            is_unsafe: false,
        };
        resolver.register_impl(impl_block);
        assert_eq!(resolver.impls.len(), 1);
    }

    #[test]
    fn test_find_impls_for_type() {
        let mut resolver = TraitResolver::new();
        let impl_block = ImplBlock {
            type_params: vec![],
            trait_ref: None,
            self_type: RustType::I32,
            where_bounds: vec![],
            methods: vec![],
            associated_types: vec![],
            associated_consts: vec![],
            is_negative: false,
            is_unsafe: false,
        };
        resolver.register_impl(impl_block);

        let found = resolver.find_impls_for_type(&RustType::I32);
        assert_eq!(found.len(), 1);

        let not_found = resolver.find_impls_for_type(&RustType::Str);
        assert!(not_found.is_empty());
    }
}

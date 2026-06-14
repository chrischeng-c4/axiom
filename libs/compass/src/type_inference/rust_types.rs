//! Rust-specific type system extensions
//!
//! This module provides Rust-specific type constructs for the Lens type system,
//! including traits, lifetimes, and Rust-specific type inference.

use super::ty::{Type, TypeVarId};

// ============================================================================
// Lifetime Types
// ============================================================================

/// Unique identifier for lifetimes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LifetimeId(pub usize);

/// Rust lifetime representation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Lifetime {
    /// Named lifetime ('a, 'b, etc.)
    Named { id: LifetimeId, name: String },
    /// Static lifetime ('static)
    Static,
    /// Anonymous/elided lifetime ('_)
    Anonymous,
    /// Inferred lifetime (not yet resolved)
    Inferred(LifetimeId),
}

impl Lifetime {
    /// Create a new named lifetime
    pub fn named(id: LifetimeId, name: impl Into<String>) -> Self {
        Lifetime::Named {
            id,
            name: name.into(),
        }
    }

    /// Check if this lifetime outlives another
    pub fn outlives(&self, other: &Lifetime) -> bool {
        match (self, other) {
            (Lifetime::Static, _) => true,
            (_, Lifetime::Static) => false,
            (Lifetime::Named { id: a, .. }, Lifetime::Named { id: b, .. }) => a == b,
            _ => false, // Conservative: unknown relationships
        }
    }
}

// ============================================================================
// Trait Types
// ============================================================================

/// Unique identifier for traits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraitId(pub usize);

/// Rust trait definition
#[derive(Debug, Clone)]
pub struct TraitDef {
    /// Trait identifier
    pub id: TraitId,
    /// Trait name
    pub name: String,
    /// Module path
    pub module: Option<String>,
    /// Generic type parameters
    pub type_params: Vec<RustTypeParam>,
    /// Supertraits (trait bounds)
    pub supertraits: Vec<TraitBound>,
    /// Associated types
    pub associated_types: Vec<AssociatedType>,
    /// Required methods
    pub required_methods: Vec<TraitMethod>,
    /// Provided methods (with default impl)
    pub provided_methods: Vec<TraitMethod>,
    /// Whether this is an auto trait (Send, Sync, etc.)
    pub is_auto: bool,
    /// Whether this is a marker trait (no methods)
    pub is_marker: bool,
}

/// Trait bound (e.g., T: Clone + Send)
#[derive(Debug, Clone, PartialEq)]
pub struct TraitBound {
    /// The trait being bounded
    pub trait_ref: TraitRef,
    /// Whether this is a negative bound (T: !Sync)
    pub is_negative: bool,
    /// Higher-ranked lifetime bounds (for<'a> T: Trait<'a>)
    pub higher_ranked_lifetimes: Vec<Lifetime>,
}

/// Reference to a trait with type arguments
#[derive(Debug, Clone, PartialEq)]
pub struct TraitRef {
    /// Trait identifier
    pub trait_id: TraitId,
    /// Trait name (for display)
    pub name: String,
    /// Type arguments
    pub type_args: Vec<RustType>,
    /// Lifetime arguments
    pub lifetime_args: Vec<Lifetime>,
}

/// Associated type in a trait
#[derive(Debug, Clone)]
pub struct AssociatedType {
    /// Name of the associated type
    pub name: String,
    /// Bounds on the associated type
    pub bounds: Vec<TraitBound>,
    /// Default type (if provided)
    pub default: Option<RustType>,
}

/// Trait method signature
#[derive(Debug, Clone)]
pub struct TraitMethod {
    /// Method name
    pub name: String,
    /// Generic parameters
    pub type_params: Vec<RustTypeParam>,
    /// Self parameter type
    pub self_param: Option<SelfParam>,
    /// Other parameters
    pub params: Vec<RustParam>,
    /// Return type
    pub return_type: RustType,
    /// Where clause bounds
    pub where_bounds: Vec<WherePredicate>,
    /// Whether this is unsafe
    pub is_unsafe: bool,
    /// Whether this is async
    pub is_async: bool,
}

/// Self parameter in a method
#[derive(Debug, Clone, PartialEq)]
pub enum SelfParam {
    /// self
    Value,
    /// &self
    Ref(Option<Lifetime>),
    /// &mut self
    RefMut(Option<Lifetime>),
    /// self: Box<Self>
    Explicit(Box<RustType>),
}

// ============================================================================
// Rust Type Extensions
// ============================================================================

/// Rust-specific type representation
#[derive(Debug, Clone, PartialEq)]
pub enum RustType {
    /// Primitive types
    Unit,
    Bool,
    Char,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    F32,
    F64,
    Str,

    /// Reference type (&T or &mut T)
    Reference {
        lifetime: Option<Lifetime>,
        mutable: bool,
        inner: Box<RustType>,
    },

    /// Raw pointer (*const T or *mut T)
    RawPointer {
        mutable: bool,
        inner: Box<RustType>,
    },

    /// Slice type ([T])
    Slice(Box<RustType>),

    /// Array type ([T; N])
    Array {
        element: Box<RustType>,
        size: usize,
    },

    /// Tuple type ((A, B, C))
    Tuple(Vec<RustType>),

    /// Function pointer (fn(A, B) -> C)
    FnPointer {
        params: Vec<RustType>,
        return_type: Box<RustType>,
        is_unsafe: bool,
        abi: Option<String>,
    },

    /// Closure type (impl Fn(A) -> B)
    Closure {
        kind: ClosureKind,
        params: Vec<RustType>,
        return_type: Box<RustType>,
    },

    /// Named type (struct, enum, type alias)
    Named {
        name: String,
        module: Option<String>,
        type_args: Vec<RustType>,
        lifetime_args: Vec<Lifetime>,
    },

    /// Trait object (dyn Trait + 'a)
    TraitObject {
        bounds: Vec<TraitBound>,
        lifetime: Option<Lifetime>,
    },

    /// Impl trait (impl Trait)
    ImplTrait {
        bounds: Vec<TraitBound>,
    },

    /// Type parameter (generic T)
    TypeParam {
        id: TypeVarId,
        name: String,
        bounds: Vec<TraitBound>,
    },

    /// Associated type projection (T::Item)
    Projection {
        base: Box<RustType>,
        trait_ref: Option<TraitRef>,
        name: String,
    },

    /// Never type (!)
    Never,

    /// Inferred type (_)
    Infer,

    /// Error type (for error recovery)
    Error,
}

/// Closure kind (Fn, FnMut, FnOnce)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClosureKind {
    Fn,
    FnMut,
    FnOnce,
}

/// Rust type parameter with bounds
#[derive(Debug, Clone)]
pub struct RustTypeParam {
    /// Parameter name
    pub name: String,
    /// Type variable ID
    pub id: TypeVarId,
    /// Trait bounds
    pub bounds: Vec<TraitBound>,
    /// Default type
    pub default: Option<RustType>,
}

/// Rust function/method parameter
#[derive(Debug, Clone)]
pub struct RustParam {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub ty: RustType,
    /// Whether this is a pattern parameter
    pub is_pattern: bool,
}

/// Where clause predicate
#[derive(Debug, Clone)]
pub enum WherePredicate {
    /// Type bound (T: Trait)
    TypeBound {
        ty: RustType,
        bounds: Vec<TraitBound>,
    },
    /// Lifetime bound ('a: 'b)
    LifetimeBound {
        lifetime: Lifetime,
        bounds: Vec<Lifetime>,
    },
    /// Type equality (T::Item = U)
    TypeEquality { projection: RustType, ty: RustType },
}

// ============================================================================
// Impl Block Types
// ============================================================================

/// Impl block representation
#[derive(Debug, Clone)]
pub struct ImplBlock {
    /// Type parameters
    pub type_params: Vec<RustTypeParam>,
    /// Trait being implemented (if trait impl)
    pub trait_ref: Option<TraitRef>,
    /// Type being implemented for
    pub self_type: RustType,
    /// Where clause
    pub where_bounds: Vec<WherePredicate>,
    /// Methods in this impl
    pub methods: Vec<ImplMethod>,
    /// Associated types
    pub associated_types: Vec<(String, RustType)>,
    /// Associated constants
    pub associated_consts: Vec<(String, RustType)>,
    /// Whether this is a negative impl
    pub is_negative: bool,
    /// Whether this is unsafe
    pub is_unsafe: bool,
}

/// Method in an impl block
#[derive(Debug, Clone)]
pub struct ImplMethod {
    /// Method name
    pub name: String,
    /// Visibility
    pub visibility: Visibility,
    /// Method signature
    pub signature: TraitMethod,
    /// Whether this is a default impl
    pub is_default: bool,
}

/// Visibility modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Crate,
    Super,
    Private,
    Restricted, // pub(in path)
}

// ============================================================================
// Rust Symbol Types
// ============================================================================

/// Rust struct definition
#[derive(Debug, Clone)]
pub struct StructDef {
    /// Struct name
    pub name: String,
    /// Module path
    pub module: Option<String>,
    /// Type parameters
    pub type_params: Vec<RustTypeParam>,
    /// Fields
    pub fields: StructFields,
    /// Where clause
    pub where_bounds: Vec<WherePredicate>,
    /// Visibility
    pub visibility: Visibility,
}

/// Struct field variants
#[derive(Debug, Clone)]
pub enum StructFields {
    /// Named fields (struct { field: Type })
    Named(Vec<StructField>),
    /// Tuple fields (struct (Type, Type))
    Tuple(Vec<RustType>),
    /// Unit struct (struct Name;)
    Unit,
}

/// Named struct field
#[derive(Debug, Clone)]
pub struct StructField {
    /// Field name
    pub name: String,
    /// Field type
    pub ty: RustType,
    /// Visibility
    pub visibility: Visibility,
}

/// Rust enum definition
#[derive(Debug, Clone)]
pub struct EnumDef {
    /// Enum name
    pub name: String,
    /// Module path
    pub module: Option<String>,
    /// Type parameters
    pub type_params: Vec<RustTypeParam>,
    /// Variants
    pub variants: Vec<EnumVariant>,
    /// Where clause
    pub where_bounds: Vec<WherePredicate>,
    /// Visibility
    pub visibility: Visibility,
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct EnumVariant {
    /// Variant name
    pub name: String,
    /// Variant fields
    pub fields: StructFields,
    /// Discriminant value (if specified)
    pub discriminant: Option<i128>,
}

// ============================================================================
// Conversion Utilities
// ============================================================================

impl RustType {
    /// Convert to the generic Type enum (for cross-language operations)
    pub fn to_generic_type(&self) -> Type {
        match self {
            RustType::Unit => Type::None,
            RustType::Bool => Type::Bool,
            RustType::I8
            | RustType::I16
            | RustType::I32
            | RustType::I64
            | RustType::I128
            | RustType::Isize
            | RustType::U8
            | RustType::U16
            | RustType::U32
            | RustType::U64
            | RustType::U128
            | RustType::Usize => Type::Int,
            RustType::F32 | RustType::F64 => Type::Float,
            RustType::Str | RustType::Char => Type::Str,
            RustType::Reference { inner, .. } => inner.to_generic_type(),
            RustType::Slice(inner) => Type::list(inner.to_generic_type()),
            RustType::Array { element, .. } => Type::list(element.to_generic_type()),
            RustType::Tuple(elements) => {
                Type::Tuple(elements.iter().map(|t| t.to_generic_type()).collect())
            }
            RustType::Named {
                name,
                module,
                type_args,
                ..
            } => Type::Instance {
                name: name.clone(),
                module: module.clone(),
                type_args: type_args.iter().map(|t| t.to_generic_type()).collect(),
            },
            RustType::Never => Type::Never,
            RustType::Infer => Type::Unknown,
            RustType::Error => Type::Error,
            _ => Type::Any, // Complex Rust types map to Any for now
        }
    }

    /// Check if this type implements a trait
    pub fn implements_trait(&self, trait_ref: &TraitRef, impls: &[ImplBlock]) -> bool {
        impls.iter().any(|impl_block| {
            // Check if this impl is for the requested trait
            if let Some(ref impl_trait) = impl_block.trait_ref {
                if impl_trait.trait_id == trait_ref.trait_id {
                    // Check if the impl's self type matches this type
                    return self.matches_impl_type(&impl_block.self_type);
                }
            }
            false
        })
    }

    /// Check if this type matches an impl's self type (for trait resolution)
    fn matches_impl_type(&self, pattern: &RustType) -> bool {
        match (pattern, self) {
            // Exact match
            (a, b) if a == b => true,

            // Type parameter in impl matches any concrete type
            (RustType::TypeParam { .. }, _) => true,

            // Named types must match name and recursively match type args
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
                        .all(|(p, c)| c.matches_impl_type(p))
            }

            // Reference types must match mutability and inner type
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
            ) => m1 == m2 && i2.matches_impl_type(i1),

            // Slices
            (RustType::Slice(e1), RustType::Slice(e2)) => e2.matches_impl_type(e1),

            // Arrays must match element type and size
            (
                RustType::Array {
                    element: e1,
                    size: s1,
                },
                RustType::Array {
                    element: e2,
                    size: s2,
                },
            ) => s1 == s2 && e2.matches_impl_type(e1),

            // Tuples must match length and element types
            (RustType::Tuple(t1), RustType::Tuple(t2)) => {
                t1.len() == t2.len()
                    && t1
                        .iter()
                        .zip(t2.iter())
                        .all(|(p, c)| c.matches_impl_type(p))
            }

            // Primitives match exactly (handled by first arm)
            _ => false,
        }
    }

    /// Get the size of this type (for arrays)
    pub fn array_size(&self) -> Option<usize> {
        match self {
            RustType::Array { size, .. } => Some(*size),
            _ => None,
        }
    }
}

impl Default for RustType {
    fn default() -> Self {
        RustType::Infer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifetime_outlives() {
        let static_lt = Lifetime::Static;
        let named_a = Lifetime::named(LifetimeId(0), "a");
        let named_b = Lifetime::named(LifetimeId(1), "b");

        assert!(static_lt.outlives(&named_a));
        assert!(static_lt.outlives(&named_b));
        assert!(!named_a.outlives(&static_lt));
        assert!(named_a.outlives(&named_a));
        assert!(!named_a.outlives(&named_b));
    }

    #[test]
    fn test_rust_type_to_generic() {
        let int_type = RustType::I32;
        assert_eq!(int_type.to_generic_type(), Type::Int);

        let str_type = RustType::Str;
        assert_eq!(str_type.to_generic_type(), Type::Str);

        let ref_type = RustType::Reference {
            lifetime: None,
            mutable: false,
            inner: Box::new(RustType::I32),
        };
        assert_eq!(ref_type.to_generic_type(), Type::Int);
    }

    #[test]
    fn test_struct_fields() {
        let struct_def = StructDef {
            name: "Point".to_string(),
            module: None,
            type_params: vec![],
            fields: StructFields::Named(vec![
                StructField {
                    name: "x".to_string(),
                    ty: RustType::F64,
                    visibility: Visibility::Public,
                },
                StructField {
                    name: "y".to_string(),
                    ty: RustType::F64,
                    visibility: Visibility::Public,
                },
            ]),
            where_bounds: vec![],
            visibility: Visibility::Public,
        };

        assert_eq!(struct_def.name, "Point");
        if let StructFields::Named(fields) = &struct_def.fields {
            assert_eq!(fields.len(), 2);
        } else {
            panic!("Expected named fields");
        }
    }

    #[test]
    fn test_implements_trait_exact_match() {
        let clone_trait = TraitRef {
            trait_id: TraitId(1),
            name: "Clone".to_string(),
            type_args: vec![],
            lifetime_args: vec![],
        };

        let impl_block = ImplBlock {
            type_params: vec![],
            trait_ref: Some(clone_trait.clone()),
            self_type: RustType::I32,
            where_bounds: vec![],
            methods: vec![],
            associated_types: vec![],
            associated_consts: vec![],
            is_negative: false,
            is_unsafe: false,
        };

        let impls = vec![impl_block];

        // i32 should implement Clone
        assert!(RustType::I32.implements_trait(&clone_trait, &impls));

        // i64 should not implement Clone (no impl for it)
        assert!(!RustType::I64.implements_trait(&clone_trait, &impls));
    }

    #[test]
    fn test_implements_trait_generic_impl() {
        use crate::type_inference::ty::TypeVarId;

        let debug_trait = TraitRef {
            trait_id: TraitId(2),
            name: "Debug".to_string(),
            type_args: vec![],
            lifetime_args: vec![],
        };

        // impl<T> Debug for Vec<T>
        let generic_impl = ImplBlock {
            type_params: vec![RustTypeParam {
                name: "T".to_string(),
                id: TypeVarId(0),
                bounds: vec![],
                default: None,
            }],
            trait_ref: Some(debug_trait.clone()),
            self_type: RustType::Named {
                name: "Vec".to_string(),
                module: None,
                type_args: vec![RustType::TypeParam {
                    id: TypeVarId(0),
                    name: "T".to_string(),
                    bounds: vec![],
                }],
                lifetime_args: vec![],
            },
            where_bounds: vec![],
            methods: vec![],
            associated_types: vec![],
            associated_consts: vec![],
            is_negative: false,
            is_unsafe: false,
        };

        let impls = vec![generic_impl];

        // Vec<i32> should implement Debug
        let vec_i32 = RustType::Named {
            name: "Vec".to_string(),
            module: None,
            type_args: vec![RustType::I32],
            lifetime_args: vec![],
        };
        assert!(vec_i32.implements_trait(&debug_trait, &impls));

        // Vec<String> should also implement Debug (generic impl)
        let vec_string = RustType::Named {
            name: "Vec".to_string(),
            module: None,
            type_args: vec![RustType::Named {
                name: "String".to_string(),
                module: None,
                type_args: vec![],
                lifetime_args: vec![],
            }],
            lifetime_args: vec![],
        };
        assert!(vec_string.implements_trait(&debug_trait, &impls));

        // HashMap should NOT implement Debug (wrong name)
        let hashmap = RustType::Named {
            name: "HashMap".to_string(),
            module: None,
            type_args: vec![RustType::I32, RustType::I32],
            lifetime_args: vec![],
        };
        assert!(!hashmap.implements_trait(&debug_trait, &impls));
    }

    #[test]
    fn test_implements_trait_no_impls() {
        let send_trait = TraitRef {
            trait_id: TraitId(3),
            name: "Send".to_string(),
            type_args: vec![],
            lifetime_args: vec![],
        };

        // Empty impls - nothing implements Send
        let impls: Vec<ImplBlock> = vec![];
        assert!(!RustType::I32.implements_trait(&send_trait, &impls));
    }

    #[test]
    fn test_implements_trait_wrong_trait() {
        let clone_trait = TraitRef {
            trait_id: TraitId(1),
            name: "Clone".to_string(),
            type_args: vec![],
            lifetime_args: vec![],
        };

        let copy_trait = TraitRef {
            trait_id: TraitId(2),
            name: "Copy".to_string(),
            type_args: vec![],
            lifetime_args: vec![],
        };

        // impl Clone for i32
        let impl_block = ImplBlock {
            type_params: vec![],
            trait_ref: Some(clone_trait.clone()),
            self_type: RustType::I32,
            where_bounds: vec![],
            methods: vec![],
            associated_types: vec![],
            associated_consts: vec![],
            is_negative: false,
            is_unsafe: false,
        };

        let impls = vec![impl_block];

        // i32 implements Clone but NOT Copy (different trait ID)
        assert!(RustType::I32.implements_trait(&clone_trait, &impls));
        assert!(!RustType::I32.implements_trait(&copy_trait, &impls));
    }
}

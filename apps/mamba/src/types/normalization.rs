use crate::parser::ast::{ParamAnnotation, SourceAnnotation, TypeExpr};
use crate::source::span::Spanned;
use crate::types::ty::TypeId;

/// Source provenance licensing a type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeProvenance {
    /// Authored concrete annotation or unresolved type (not explicit Any).
    Explicit,
    /// Explicitly authored `Any` or `typing.Any`.
    ExplicitAny,
    /// Omitted annotation with successful inference.
    Inferred { inference_path: String },
    /// Omitted annotation with failed inference.
    ImplicitUnknown { inference_path: String },
}

/// Declared type aggregate containing source annotation presence, normalized type ID (if resolved), and provenance.
#[derive(Debug, Clone, PartialEq)]
pub struct DeclaredType {
    source: SourceAnnotation,
    normalized: Option<TypeId>,
    provenance: TypeProvenance,
}

impl DeclaredType {
    /// Construct a `DeclaredType` from authored type expression syntax.
    /// Derived provenance is automatically computed from the source syntax.
    pub(crate) fn from_authored(syntax: Spanned<TypeExpr>, normalized: Option<TypeId>) -> Self {
        let provenance = classify_provenance(&syntax.node);
        Self {
            source: SourceAnnotation::Authored(syntax),
            normalized,
            provenance,
        }
    }

    /// Construct a `DeclaredType` from a `SourceAnnotation` (or `ParamAnnotation`).
    /// Returns `None` for omitted annotations, ensuring omitted parameters never enter
    /// the authored-only constructor.
    pub(crate) fn from_annotation(annotation: &SourceAnnotation, normalized: Option<TypeId>) -> Option<Self> {
        match annotation {
            SourceAnnotation::Authored(syntax) => Some(Self::from_authored(syntax.clone(), normalized)),
            SourceAnnotation::Omitted => None,
        }
    }

    /// Construct a `DeclaredType` for an omitted annotation with successful inference.
    pub(crate) fn from_inferred(inference_path: impl Into<String>, normalized: TypeId) -> Self {
        Self {
            source: SourceAnnotation::Omitted,
            normalized: Some(normalized),
            provenance: TypeProvenance::Inferred {
                inference_path: inference_path.into(),
            },
        }
    }

    /// Construct a `DeclaredType` for an omitted annotation with failed inference.
    pub(crate) fn from_implicit_unknown(inference_path: impl Into<String>) -> Self {
        Self {
            source: SourceAnnotation::Omitted,
            normalized: None,
            provenance: TypeProvenance::ImplicitUnknown {
                inference_path: inference_path.into(),
            },
        }
    }

    pub fn source(&self) -> &SourceAnnotation {
        &self.source
    }

    pub fn normalized(&self) -> Option<TypeId> {
        self.normalized
    }

    pub fn provenance(&self) -> &TypeProvenance {
        &self.provenance
    }

    pub fn dynamic_boundary_license(&self) -> Option<DynamicBoundaryLicense> {
        DynamicBoundaryLicense::from_provenance(&self.provenance)
    }
}

/// Closed value object representing a dynamic boundary license granted by explicit `Any`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DynamicBoundaryLicense {
    ExplicitAny,
}

impl DynamicBoundaryLicense {
    /// Derive a boundary license from stored type provenance.
    pub fn from_provenance(provenance: &TypeProvenance) -> Option<Self> {
        match provenance {
            TypeProvenance::ExplicitAny => Some(DynamicBoundaryLicense::ExplicitAny),
            TypeProvenance::Explicit
            | TypeProvenance::Inferred { .. }
            | TypeProvenance::ImplicitUnknown { .. } => None,
        }
    }
}

/// Classify whether syntax is explicit `Any` or `typing.Any`.
pub(crate) fn is_explicit_any_syntax(ty: &TypeExpr) -> bool {
    match ty {
        TypeExpr::Named(name) => name == "Any" || name == "typing.Any",
        _ => false,
    }
}

/// Helper function to classify provenance for an authored `TypeExpr`.
pub(crate) fn classify_provenance(ty: &TypeExpr) -> TypeProvenance {
    if is_explicit_any_syntax(ty) {
        TypeProvenance::ExplicitAny
    } else {
        TypeProvenance::Explicit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::span::{FileId, Span};

    #[test]
    fn test_is_explicit_any_syntax() {
        assert!(is_explicit_any_syntax(&TypeExpr::Named("Any".to_string())));
        assert!(is_explicit_any_syntax(&TypeExpr::Named("typing.Any".to_string())));
        assert!(!is_explicit_any_syntax(&TypeExpr::Named("int".to_string())));
        assert!(!is_explicit_any_syntax(&TypeExpr::Named("object".to_string())));
        assert!(!is_explicit_any_syntax(&TypeExpr::Named("NonExistent".to_string())));
    }

    #[test]
    fn test_declared_type_omitted_returns_none() {
        let omitted = ParamAnnotation::Omitted;
        assert!(DeclaredType::from_annotation(&omitted, Some(TypeId(0))).is_none());
    }

    #[test]
    fn test_declared_type_authored_classification() {
        let dummy_span = Span::new(FileId(0), 0, 5);

        let any_syntax = Spanned::new(TypeExpr::Named("Any".to_string()), dummy_span);
        let decl_any = DeclaredType::from_authored(any_syntax, Some(TypeId(1)));
        assert_eq!(*decl_any.provenance(), TypeProvenance::ExplicitAny);

        let typing_any_syntax = Spanned::new(TypeExpr::Named("typing.Any".to_string()), dummy_span);
        let decl_typing_any = DeclaredType::from_authored(typing_any_syntax, Some(TypeId(1)));
        assert_eq!(*decl_typing_any.provenance(), TypeProvenance::ExplicitAny);

        let obj_syntax = Spanned::new(TypeExpr::Named("object".to_string()), dummy_span);
        let decl_obj = DeclaredType::from_authored(obj_syntax, Some(TypeId(1)));
        assert_eq!(*decl_obj.provenance(), TypeProvenance::Explicit);

        let int_syntax = Spanned::new(TypeExpr::Named("int".to_string()), dummy_span);
        let decl_int = DeclaredType::from_authored(int_syntax, Some(TypeId(2)));
        assert_eq!(*decl_int.provenance(), TypeProvenance::Explicit);
    }

    #[test]
    fn test_declared_type_inferred_invariants() {
        let decl = DeclaredType::from_inferred("local_binding -> list_literal", TypeId(42));
        assert_eq!(*decl.source(), SourceAnnotation::Omitted);
        assert_eq!(decl.normalized(), Some(TypeId(42)));
        assert_eq!(
            *decl.provenance(),
            TypeProvenance::Inferred {
                inference_path: "local_binding -> list_literal".to_string()
            }
        );
    }

    #[test]
    fn test_declared_type_implicit_unknown_invariants() {
        let decl = DeclaredType::from_implicit_unknown("local_binding -> list_literal -> element");
        assert_eq!(*decl.source(), SourceAnnotation::Omitted);
        assert_eq!(decl.normalized(), None);
        assert_eq!(
            *decl.provenance(),
            TypeProvenance::ImplicitUnknown {
                inference_path: "local_binding -> list_literal -> element".to_string()
            }
        );
    }

    #[test]
    fn test_dynamic_boundary_license_exhaustive_derivation() {
        assert_eq!(
            DynamicBoundaryLicense::from_provenance(&TypeProvenance::ExplicitAny),
            Some(DynamicBoundaryLicense::ExplicitAny)
        );
        assert_eq!(
            DynamicBoundaryLicense::from_provenance(&TypeProvenance::Explicit),
            None
        );
        assert_eq!(
            DynamicBoundaryLicense::from_provenance(&TypeProvenance::Inferred {
                inference_path: "test".to_string()
            }),
            None
        );
        assert_eq!(
            DynamicBoundaryLicense::from_provenance(&TypeProvenance::ImplicitUnknown {
                inference_path: "test".to_string()
            }),
            None
        );
    }

    #[test]
    fn test_declared_type_dynamic_boundary_license() {
        let dummy_span = Span::new(FileId(0), 0, 5);

        let any_syntax = Spanned::new(TypeExpr::Named("Any".to_string()), dummy_span);
        let decl_any = DeclaredType::from_authored(any_syntax, Some(TypeId(1)));
        assert_eq!(decl_any.dynamic_boundary_license(), Some(DynamicBoundaryLicense::ExplicitAny));

        let typing_any_syntax = Spanned::new(TypeExpr::Named("typing.Any".to_string()), dummy_span);
        let decl_typing_any = DeclaredType::from_authored(typing_any_syntax, Some(TypeId(1)));
        assert_eq!(decl_typing_any.dynamic_boundary_license(), Some(DynamicBoundaryLicense::ExplicitAny));

        let int_syntax = Spanned::new(TypeExpr::Named("int".to_string()), dummy_span);
        let decl_int = DeclaredType::from_authored(int_syntax, Some(TypeId(2)));
        assert_eq!(decl_int.dynamic_boundary_license(), None);

        let decl_inferred = DeclaredType::from_inferred("inferred_path", TypeId(42));
        assert_eq!(decl_inferred.dynamic_boundary_license(), None);

        let decl_unknown = DeclaredType::from_implicit_unknown("unknown_path");
        assert_eq!(decl_unknown.dynamic_boundary_license(), None);
    }
}

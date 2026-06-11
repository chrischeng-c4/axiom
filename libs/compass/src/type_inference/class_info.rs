//! Class information for type inference

use std::collections::HashMap;

use super::ty::{Type, TypeVarId, Variance};

/// Information about a generic type parameter on a class
#[derive(Debug, Clone)]
pub struct GenericParam {
    /// TypeVar ID
    pub id: TypeVarId,
    /// Parameter name (e.g., "T", "K", "V")
    pub name: String,
    /// Variance of this type parameter
    pub variance: Variance,
    /// Optional upper bound
    pub bound: Option<Type>,
    /// Type constraints (if any)
    pub constraints: Vec<Type>,
}

impl GenericParam {
    pub fn new(id: TypeVarId, name: String) -> Self {
        Self {
            id,
            name,
            variance: Variance::Invariant,
            bound: None,
            constraints: Vec::new(),
        }
    }

    pub fn with_variance(mut self, variance: Variance) -> Self {
        self.variance = variance;
        self
    }

    pub fn with_bound(mut self, bound: Type) -> Self {
        self.bound = Some(bound);
        self
    }

    pub fn with_constraints(mut self, constraints: Vec<Type>) -> Self {
        self.constraints = constraints;
        self
    }
}

/// Information about a class definition
#[derive(Debug, Clone, Default)]
pub struct ClassInfo {
    /// Class name
    pub name: String,
    /// Base classes (may include generic instantiations like "Generic[T]")
    pub bases: Vec<String>,
    /// Generic type parameters (in declaration order)
    pub generic_params: Vec<GenericParam>,
    /// Instance attributes (name -> type)
    pub attributes: HashMap<String, Type>,
    /// Methods (name -> callable type)
    pub methods: HashMap<String, Type>,
    /// Class variables (name -> type)
    pub class_vars: HashMap<String, Type>,
    /// Whether this class is a Protocol
    pub is_protocol: bool,
    /// Whether this class is abstract
    pub is_abstract: bool,
}

impl ClassInfo {
    pub fn new(name: String) -> Self {
        Self {
            name,
            bases: Vec::new(),
            generic_params: Vec::new(),
            attributes: HashMap::new(),
            methods: HashMap::new(),
            class_vars: HashMap::new(),
            is_protocol: false,
            is_abstract: false,
        }
    }

    /// Check if this class is generic (has type parameters)
    pub fn is_generic(&self) -> bool {
        !self.generic_params.is_empty()
    }

    /// Get the number of type parameters
    pub fn arity(&self) -> usize {
        self.generic_params.len()
    }

    /// Get a type parameter by name
    pub fn get_type_param(&self, name: &str) -> Option<&GenericParam> {
        self.generic_params.iter().find(|p| p.name == name)
    }

    /// Get a type parameter by index
    pub fn get_type_param_by_index(&self, index: usize) -> Option<&GenericParam> {
        self.generic_params.get(index)
    }

    /// Get variance for a type parameter by index
    pub fn variance_at(&self, index: usize) -> Variance {
        self.generic_params
            .get(index)
            .map(|p| p.variance)
            .unwrap_or(Variance::Invariant)
    }

    /// Add a generic type parameter
    pub fn add_type_param(&mut self, param: GenericParam) {
        self.generic_params.push(param);
    }

    /// Get attribute type (checks instance attrs, then methods, then class vars)
    pub fn get_attribute(&self, name: &str) -> Option<&Type> {
        self.attributes
            .get(name)
            .or_else(|| self.methods.get(name))
            .or_else(|| self.class_vars.get(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_info_new() {
        let info = ClassInfo::new("MyClass".to_string());
        assert_eq!(info.name, "MyClass");
        assert!(!info.is_generic());
        assert_eq!(info.arity(), 0);
    }

    #[test]
    fn test_generic_class() {
        let mut info = ClassInfo::new("Container".to_string());
        let param =
            GenericParam::new(TypeVarId(0), "T".to_string()).with_variance(Variance::Covariant);
        info.add_type_param(param);

        assert!(info.is_generic());
        assert_eq!(info.arity(), 1);
        assert_eq!(info.variance_at(0), Variance::Covariant);
        assert!(info.get_type_param("T").is_some());
    }

    #[test]
    fn test_multiple_type_params() {
        let mut info = ClassInfo::new("Dict".to_string());
        info.add_type_param(GenericParam::new(TypeVarId(0), "K".to_string()));
        info.add_type_param(
            GenericParam::new(TypeVarId(1), "V".to_string()).with_variance(Variance::Covariant),
        );

        assert_eq!(info.arity(), 2);
        assert_eq!(info.variance_at(0), Variance::Invariant);
        assert_eq!(info.variance_at(1), Variance::Covariant);
    }
}

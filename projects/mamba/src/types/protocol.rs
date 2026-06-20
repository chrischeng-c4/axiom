use super::context::TypeContext;
use super::ty::TypeId;
/// Protocol types (structural subtyping) for Mamba (#314 R2).
///
/// Implements Python's Protocol pattern where an object satisfies a protocol
/// if it implements all required methods/attributes, regardless of its
/// inheritance hierarchy.
///
/// Example:
///   class Drawable(Protocol):
///       def draw(self) -> None: ...
///
///   class Circle:
///       def draw(self) -> None: ...
///
///   # Circle satisfies Drawable (structural match)
use std::collections::HashMap;

/// A protocol definition: required method signatures.
#[derive(Debug, Clone)]
pub struct Protocol {
    pub name: String,
    /// Required methods: name → (param types, return type)
    pub methods: HashMap<String, MethodSig>,
    /// Required attributes: name → type
    pub attrs: HashMap<String, TypeId>,
    /// Whether this protocol is runtime-checkable
    pub runtime_checkable: bool,
}

/// Method signature within a protocol.
#[derive(Debug, Clone)]
pub struct MethodSig {
    pub params: Vec<TypeId>,
    pub return_type: TypeId,
}

/// Protocol registry for the type checker.
#[derive(Debug, Default)]
pub struct ProtocolRegistry {
    protocols: HashMap<String, Protocol>,
}

impl ProtocolRegistry {
    pub fn new() -> Self {
        Self {
            protocols: HashMap::new(),
        }
    }

    /// Register a new protocol.
    pub fn register(&mut self, protocol: Protocol) {
        self.protocols.insert(protocol.name.clone(), protocol);
    }

    /// Get a protocol by name.
    pub fn get(&self, name: &str) -> Option<&Protocol> {
        self.protocols.get(name)
    }

    /// Check if a class structurally satisfies a protocol.
    ///
    /// Returns a list of missing methods/attributes. Empty = satisfies protocol.
    pub fn check_conformance(
        &self,
        protocol_name: &str,
        class_methods: &HashMap<String, MethodSig>,
        class_attrs: &HashMap<String, TypeId>,
        tcx: &TypeContext,
    ) -> Vec<ProtocolViolation> {
        let mut violations = Vec::new();

        let protocol = match self.protocols.get(protocol_name) {
            Some(p) => p,
            None => {
                return vec![ProtocolViolation::ProtocolNotFound(
                    protocol_name.to_string(),
                )]
            }
        };

        // Check required methods
        for (method_name, required_sig) in &protocol.methods {
            match class_methods.get(method_name) {
                None => {
                    violations.push(ProtocolViolation::MissingMethod(method_name.clone()));
                }
                Some(actual_sig) => {
                    // Check parameter count matches
                    if actual_sig.params.len() != required_sig.params.len() {
                        violations.push(ProtocolViolation::MethodSignatureMismatch {
                            method: method_name.clone(),
                            reason: format!(
                                "expected {} params, got {}",
                                required_sig.params.len(),
                                actual_sig.params.len()
                            ),
                        });
                    } else {
                        // Check parameter type compatibility (contravariant:
                        // actual params must be supertypes of required params)
                        for (i, (req, act)) in required_sig
                            .params
                            .iter()
                            .zip(actual_sig.params.iter())
                            .enumerate()
                        {
                            if !tcx.is_subtype(*req, *act) {
                                violations.push(ProtocolViolation::MethodSignatureMismatch {
                                    method: method_name.clone(),
                                    reason: format!("parameter {} has incompatible type", i),
                                });
                            }
                        }
                    }
                    // Check return type compatibility
                    if !tcx.is_subtype(actual_sig.return_type, required_sig.return_type) {
                        violations.push(ProtocolViolation::MethodSignatureMismatch {
                            method: method_name.clone(),
                            reason: "incompatible return type".to_string(),
                        });
                    }
                }
            }
        }

        // Check required attributes
        for (attr_name, required_ty) in &protocol.attrs {
            match class_attrs.get(attr_name) {
                None => {
                    violations.push(ProtocolViolation::MissingAttribute(attr_name.clone()));
                }
                Some(actual_ty) => {
                    if !tcx.is_subtype(*actual_ty, *required_ty) {
                        violations.push(ProtocolViolation::AttributeTypeMismatch {
                            attr: attr_name.clone(),
                        });
                    }
                }
            }
        }

        violations
    }

    /// Quick check: does a class satisfy a protocol?
    pub fn satisfies(
        &self,
        protocol_name: &str,
        class_methods: &HashMap<String, MethodSig>,
        class_attrs: &HashMap<String, TypeId>,
        tcx: &TypeContext,
    ) -> bool {
        self.check_conformance(protocol_name, class_methods, class_attrs, tcx)
            .is_empty()
    }
}

/// A violation when checking protocol conformance.
#[derive(Debug, Clone)]
pub enum ProtocolViolation {
    ProtocolNotFound(String),
    MissingMethod(String),
    MissingAttribute(String),
    MethodSignatureMismatch { method: String, reason: String },
    AttributeTypeMismatch { attr: String },
}

impl std::fmt::Display for ProtocolViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProtocolNotFound(name) => write!(f, "protocol '{name}' not found"),
            Self::MissingMethod(name) => write!(f, "missing method '{name}'"),
            Self::MissingAttribute(name) => write!(f, "missing attribute '{name}'"),
            Self::MethodSignatureMismatch { method, reason } => {
                write!(f, "method '{method}' signature mismatch: {reason}")
            }
            Self::AttributeTypeMismatch { attr } => {
                write!(f, "attribute '{attr}' has incompatible type")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_satisfaction() {
        let tcx = TypeContext::new();
        let none_ty = tcx.none();

        let mut registry = ProtocolRegistry::new();

        // Define Drawable protocol with draw() → None
        registry.register(Protocol {
            name: "Drawable".to_string(),
            methods: {
                let mut m = HashMap::new();
                m.insert(
                    "draw".to_string(),
                    MethodSig {
                        params: vec![], // self is implicit
                        return_type: none_ty,
                    },
                );
                m
            },
            attrs: HashMap::new(),
            runtime_checkable: false,
        });

        // Circle has draw() → None
        let mut circle_methods = HashMap::new();
        circle_methods.insert(
            "draw".to_string(),
            MethodSig {
                params: vec![],
                return_type: none_ty,
            },
        );

        assert!(registry.satisfies("Drawable", &circle_methods, &HashMap::new(), &tcx));

        // Square does NOT have draw()
        let square_methods = HashMap::new();
        assert!(!registry.satisfies("Drawable", &square_methods, &HashMap::new(), &tcx));
    }

    #[test]
    fn test_protocol_violations() {
        let tcx = TypeContext::new();
        let _none_ty = tcx.none();
        let int_ty = tcx.int();

        let mut registry = ProtocolRegistry::new();
        registry.register(Protocol {
            name: "Comparable".to_string(),
            methods: {
                let mut m = HashMap::new();
                m.insert(
                    "__lt__".to_string(),
                    MethodSig {
                        params: vec![int_ty],
                        return_type: tcx.bool(),
                    },
                );
                m
            },
            attrs: {
                let mut a = HashMap::new();
                a.insert("value".to_string(), int_ty);
                a
            },
            runtime_checkable: false,
        });

        // Class with no methods/attrs
        let violations =
            registry.check_conformance("Comparable", &HashMap::new(), &HashMap::new(), &tcx);
        assert_eq!(violations.len(), 2); // missing method + missing attr

        // Class with __lt__ but no value attr
        let mut methods = HashMap::new();
        methods.insert(
            "__lt__".to_string(),
            MethodSig {
                params: vec![int_ty],
                return_type: tcx.bool(),
            },
        );
        let violations = registry.check_conformance("Comparable", &methods, &HashMap::new(), &tcx);
        assert_eq!(violations.len(), 1); // missing attr only
    }

    #[test]
    fn test_protocol_not_found() {
        let tcx = TypeContext::new();
        let registry = ProtocolRegistry::new();

        let violations =
            registry.check_conformance("NonExistent", &HashMap::new(), &HashMap::new(), &tcx);
        assert_eq!(violations.len(), 1);
        match &violations[0] {
            ProtocolViolation::ProtocolNotFound(name) => {
                assert_eq!(name, "NonExistent");
            }
            _ => panic!("expected ProtocolNotFound"),
        }
    }

    #[test]
    fn test_protocol_satisfies_returns_false_for_missing() {
        let tcx = TypeContext::new();
        let registry = ProtocolRegistry::new();
        // Unknown protocol → not satisfied
        assert!(!registry.satisfies("Unknown", &HashMap::new(), &HashMap::new(), &tcx));
    }

    #[test]
    fn test_protocol_register_and_get() {
        let mut registry = ProtocolRegistry::new();
        assert!(registry.get("Foo").is_none());

        registry.register(Protocol {
            name: "Foo".to_string(),
            methods: HashMap::new(),
            attrs: HashMap::new(),
            runtime_checkable: true,
        });

        let p = registry.get("Foo").unwrap();
        assert_eq!(p.name, "Foo");
        assert!(p.runtime_checkable);
        assert!(p.methods.is_empty());
    }

    #[test]
    fn test_protocol_empty_satisfies() {
        let tcx = TypeContext::new();
        let mut registry = ProtocolRegistry::new();

        // A protocol with no requirements is satisfied by anything
        registry.register(Protocol {
            name: "Empty".to_string(),
            methods: HashMap::new(),
            attrs: HashMap::new(),
            runtime_checkable: false,
        });

        assert!(registry.satisfies("Empty", &HashMap::new(), &HashMap::new(), &tcx));
    }

    #[test]
    fn test_protocol_param_count_mismatch() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let _none_ty = tcx.none();

        let mut registry = ProtocolRegistry::new();
        registry.register(Protocol {
            name: "Sizable".to_string(),
            methods: {
                let mut m = HashMap::new();
                m.insert(
                    "size".to_string(),
                    MethodSig {
                        params: vec![], // no params
                        return_type: int_ty,
                    },
                );
                m
            },
            attrs: HashMap::new(),
            runtime_checkable: false,
        });

        // Method has wrong param count
        let mut methods = HashMap::new();
        methods.insert(
            "size".to_string(),
            MethodSig {
                params: vec![int_ty], // 1 param instead of 0
                return_type: int_ty,
            },
        );

        let violations = registry.check_conformance("Sizable", &methods, &HashMap::new(), &tcx);
        assert!(violations.iter().any(|v| matches!(
            v,
            ProtocolViolation::MethodSignatureMismatch { method, reason }
            if method == "size" && reason.contains("params")
        )));
    }

    #[test]
    fn test_protocol_return_type_mismatch() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let str_ty = tcx.str();

        let mut registry = ProtocolRegistry::new();
        registry.register(Protocol {
            name: "Nameable".to_string(),
            methods: {
                let mut m = HashMap::new();
                m.insert(
                    "name".to_string(),
                    MethodSig {
                        params: vec![],
                        return_type: str_ty,
                    },
                );
                m
            },
            attrs: HashMap::new(),
            runtime_checkable: false,
        });

        // Method returns int instead of str
        let mut methods = HashMap::new();
        methods.insert(
            "name".to_string(),
            MethodSig {
                params: vec![],
                return_type: int_ty, // wrong return
            },
        );

        let violations = registry.check_conformance("Nameable", &methods, &HashMap::new(), &tcx);
        assert!(violations.iter().any(|v| matches!(
            v,
            ProtocolViolation::MethodSignatureMismatch { reason, .. }
            if reason.contains("return type")
        )));
    }

    #[test]
    fn test_protocol_attribute_type_mismatch() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let str_ty = tcx.str();

        let mut registry = ProtocolRegistry::new();
        registry.register(Protocol {
            name: "HasName".to_string(),
            methods: HashMap::new(),
            attrs: {
                let mut a = HashMap::new();
                a.insert("name".to_string(), str_ty);
                a
            },
            runtime_checkable: false,
        });

        // Attribute has wrong type
        let mut attrs = HashMap::new();
        attrs.insert("name".to_string(), int_ty); // int, not str

        let violations = registry.check_conformance("HasName", &HashMap::new(), &attrs, &tcx);
        assert!(violations.iter().any(|v| matches!(
            v, ProtocolViolation::AttributeTypeMismatch { attr } if attr == "name"
        )));
    }

    #[test]
    fn test_protocol_full_conformance() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let bool_ty = tcx.bool();

        let mut registry = ProtocolRegistry::new();
        registry.register(Protocol {
            name: "Identifiable".to_string(),
            methods: {
                let mut m = HashMap::new();
                m.insert(
                    "id".to_string(),
                    MethodSig {
                        params: vec![],
                        return_type: int_ty,
                    },
                );
                m
            },
            attrs: {
                let mut a = HashMap::new();
                a.insert("active".to_string(), bool_ty);
                a
            },
            runtime_checkable: false,
        });

        // Full conformance
        let mut methods = HashMap::new();
        methods.insert(
            "id".to_string(),
            MethodSig {
                params: vec![],
                return_type: int_ty,
            },
        );
        let mut attrs = HashMap::new();
        attrs.insert("active".to_string(), bool_ty);

        assert!(registry.satisfies("Identifiable", &methods, &attrs, &tcx));
    }

    #[test]
    fn test_violation_display() {
        let v1 = ProtocolViolation::ProtocolNotFound("X".into());
        assert_eq!(format!("{v1}"), "protocol 'X' not found");

        let v2 = ProtocolViolation::MissingMethod("foo".into());
        assert_eq!(format!("{v2}"), "missing method 'foo'");

        let v3 = ProtocolViolation::MissingAttribute("bar".into());
        assert_eq!(format!("{v3}"), "missing attribute 'bar'");

        let v4 = ProtocolViolation::MethodSignatureMismatch {
            method: "baz".into(),
            reason: "wrong params".into(),
        };
        assert_eq!(
            format!("{v4}"),
            "method 'baz' signature mismatch: wrong params"
        );

        let v5 = ProtocolViolation::AttributeTypeMismatch { attr: "qux".into() };
        assert_eq!(format!("{v5}"), "attribute 'qux' has incompatible type");
    }

    #[test]
    fn test_protocol_register_overwrites() {
        let mut registry = ProtocolRegistry::new();
        registry.register(Protocol {
            name: "P".to_string(),
            methods: HashMap::new(),
            attrs: HashMap::new(),
            runtime_checkable: false,
        });
        assert!(!registry.get("P").unwrap().runtime_checkable);

        registry.register(Protocol {
            name: "P".to_string(),
            methods: HashMap::new(),
            attrs: HashMap::new(),
            runtime_checkable: true,
        });
        assert!(registry.get("P").unwrap().runtime_checkable);
    }

    #[test]
    fn test_protocol_extra_methods_ok() {
        let tcx = TypeContext::new();
        let none_ty = tcx.none();

        let mut registry = ProtocolRegistry::new();
        registry.register(Protocol {
            name: "Drawable".to_string(),
            methods: {
                let mut m = HashMap::new();
                m.insert(
                    "draw".to_string(),
                    MethodSig {
                        params: vec![],
                        return_type: none_ty,
                    },
                );
                m
            },
            attrs: HashMap::new(),
            runtime_checkable: false,
        });

        // Class has draw() AND extra resize() — should still satisfy
        let mut methods = HashMap::new();
        methods.insert(
            "draw".to_string(),
            MethodSig {
                params: vec![],
                return_type: none_ty,
            },
        );
        methods.insert(
            "resize".to_string(),
            MethodSig {
                params: vec![tcx.int(), tcx.int()],
                return_type: none_ty,
            },
        );

        assert!(registry.satisfies("Drawable", &methods, &HashMap::new(), &tcx));
    }
}

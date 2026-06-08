use std::collections::HashMap;
use super::ty::{Ty, TypeId, TypeVarId};

/// Type variable info: optional upper bound and type constraints (#242).
#[derive(Debug, Clone)]
pub struct TypeVarInfo {
    pub name: String,
    pub bound: Option<TypeId>,
    pub constraints: Vec<TypeId>,
}

/// Interner and registry for all types used during compilation.
#[derive(Debug)]
pub struct TypeContext {
    types: Vec<Ty>,
    /// Type alias registry: name → resolved TypeId (#241).
    type_aliases: HashMap<String, TypeId>,
    /// Type variable info registry (#242).
    type_vars: Vec<TypeVarInfo>,
}

impl TypeContext {
    pub fn new() -> Self {
        let mut ctx = Self {
            types: Vec::new(),
            type_aliases: HashMap::new(),
            type_vars: Vec::new(),
        };
        // Pre-register primitive types at known positions
        ctx.intern(Ty::Never);   // TypeId(0)
        ctx.intern(Ty::None);    // TypeId(1)
        ctx.intern(Ty::Bool);    // TypeId(2)
        ctx.intern(Ty::Int);     // TypeId(3)
        ctx.intern(Ty::Float);   // TypeId(4)
        ctx.intern(Ty::Str);     // TypeId(5)
        ctx.intern(Ty::Error);   // TypeId(6)
        ctx.intern(Ty::Any);     // TypeId(7) — #240
        ctx
    }

    pub fn intern(&mut self, ty: Ty) -> TypeId {
        // Check if already interned (for primitives)
        for (i, existing) in self.types.iter().enumerate() {
            if existing == &ty {
                return TypeId(i as u32);
            }
        }
        let id = TypeId(self.types.len() as u32);
        self.types.push(ty);
        id
    }

    pub fn get(&self, id: TypeId) -> &Ty {
        &self.types[id.0 as usize]
    }

    /// Look up an already-interned type without mutating. Returns None if not found.
    pub fn find(&self, ty: &Ty) -> Option<TypeId> {
        self.types.iter().position(|t| t == ty).map(|i| TypeId(i as u32))
    }

    // Well-known type IDs
    pub fn never(&self) -> TypeId { TypeId(0) }
    pub fn none(&self) -> TypeId { TypeId(1) }
    pub fn bool(&self) -> TypeId { TypeId(2) }
    pub fn int(&self) -> TypeId { TypeId(3) }
    pub fn float(&self) -> TypeId { TypeId(4) }
    pub fn str(&self) -> TypeId { TypeId(5) }
    pub fn error(&self) -> TypeId { TypeId(6) }
    pub fn any(&self) -> TypeId { TypeId(7) }

    // --- Type aliases (#241) ---

    pub fn register_alias(&mut self, name: String, ty: TypeId) {
        self.type_aliases.insert(name, ty);
    }

    pub fn resolve_alias(&self, name: &str) -> Option<TypeId> {
        self.type_aliases.get(name).copied()
    }

    /// Remove a type alias (for scoped type parameter cleanup).
    pub fn unregister_alias(&mut self, name: &str) {
        self.type_aliases.remove(name);
    }

    // --- Type variables (#242) ---

    pub fn new_type_var(&mut self, name: String, bound: Option<TypeId>, constraints: Vec<TypeId>) -> TypeVarId {
        let id = TypeVarId(self.type_vars.len() as u32);
        self.type_vars.push(TypeVarInfo { name, bound, constraints });
        id
    }

    pub fn get_type_var(&self, id: TypeVarId) -> &TypeVarInfo {
        &self.type_vars[id.0 as usize]
    }

    // --- Subtype checking ---

    /// Check if `sub` is a subtype of `sup` (simplified).
    pub fn is_subtype(&self, sub: TypeId, sup: TypeId) -> bool {
        if sub == sup { return true; }

        let sub_ty = self.get(sub);
        let sup_ty = self.get(sup);

        // Any is compatible with everything
        if matches!(sup_ty, Ty::Any) || matches!(sub_ty, Ty::Any) {
            return true;
        }

        // Never is subtype of everything
        if matches!(sub_ty, Ty::Never) { return true; }

        // int is subtype of float (numeric widening)
        if matches!(sub_ty, Ty::Int) && matches!(sup_ty, Ty::Float) {
            return true;
        }

        // bool is subtype of int
        if matches!(sub_ty, Ty::Bool) && matches!(sup_ty, Ty::Int) {
            return true;
        }

        // Union: sub is subtype of Union if sub is subtype of any variant
        if let Ty::Union(variants) = sup_ty {
            return variants.iter().any(|v| self.is_subtype(sub, *v));
        }

        // Sub union: all variants must be subtypes of sup
        if let Ty::Union(variants) = sub_ty {
            return variants.iter().all(|v| self.is_subtype(*v, sup));
        }

        false
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_has_primitives() {
        let tcx = TypeContext::new();
        assert_eq!(*tcx.get(TypeId(0)), Ty::Never);
        assert_eq!(*tcx.get(TypeId(1)), Ty::None);
        assert_eq!(*tcx.get(TypeId(2)), Ty::Bool);
        assert_eq!(*tcx.get(TypeId(3)), Ty::Int);
        assert_eq!(*tcx.get(TypeId(4)), Ty::Float);
        assert_eq!(*tcx.get(TypeId(5)), Ty::Str);
        assert_eq!(*tcx.get(TypeId(6)), Ty::Error);
        assert_eq!(*tcx.get(TypeId(7)), Ty::Any);
    }

    #[test]
    fn test_well_known_ids() {
        let tcx = TypeContext::new();
        assert_eq!(tcx.never(), TypeId(0));
        assert_eq!(tcx.none(), TypeId(1));
        assert_eq!(tcx.bool(), TypeId(2));
        assert_eq!(tcx.int(), TypeId(3));
        assert_eq!(tcx.float(), TypeId(4));
        assert_eq!(tcx.str(), TypeId(5));
        assert_eq!(tcx.error(), TypeId(6));
        assert_eq!(tcx.any(), TypeId(7));
    }

    #[test]
    fn test_intern_dedup_primitives() {
        let mut tcx = TypeContext::new();
        // Interning a primitive again should return the same ID
        let int1 = tcx.intern(Ty::Int);
        let int2 = tcx.intern(Ty::Int);
        assert_eq!(int1, int2);
        assert_eq!(int1, tcx.int());
    }

    #[test]
    fn test_intern_compound_types() {
        let mut tcx = TypeContext::new();
        let int_ty = tcx.int();
        let list_int = tcx.intern(Ty::List(int_ty));
        // Should be a new type (not a primitive slot)
        assert!(list_int.0 >= 8);
        assert_eq!(*tcx.get(list_int), Ty::List(int_ty));
    }

    #[test]
    fn test_intern_dedup_compound() {
        let mut tcx = TypeContext::new();
        let int_ty = tcx.int();
        let list1 = tcx.intern(Ty::List(int_ty));
        let list2 = tcx.intern(Ty::List(int_ty));
        assert_eq!(list1, list2);
    }

    #[test]
    fn test_intern_distinct_compound() {
        let mut tcx = TypeContext::new();
        let int_ty = tcx.int();
        let str_ty = tcx.str();
        let list_int = tcx.intern(Ty::List(int_ty));
        let list_str = tcx.intern(Ty::List(str_ty));
        assert_ne!(list_int, list_str);
    }

    #[test]
    fn test_type_alias_register_and_resolve() {
        let mut tcx = TypeContext::new();
        let int_ty = tcx.int();
        tcx.register_alias("MyInt".to_string(), int_ty);
        assert_eq!(tcx.resolve_alias("MyInt"), Some(int_ty));
        assert_eq!(tcx.resolve_alias("Unknown"), None);
    }

    #[test]
    fn test_type_alias_unregister() {
        let mut tcx = TypeContext::new();
        let int_ty = tcx.int();
        tcx.register_alias("Temp".to_string(), int_ty);
        assert!(tcx.resolve_alias("Temp").is_some());
        tcx.unregister_alias("Temp");
        assert!(tcx.resolve_alias("Temp").is_none());
    }

    #[test]
    fn test_type_alias_overwrite() {
        let mut tcx = TypeContext::new();
        let int_ty = tcx.int();
        let str_ty = tcx.str();
        tcx.register_alias("X".to_string(), int_ty);
        assert_eq!(tcx.resolve_alias("X"), Some(int_ty));
        tcx.register_alias("X".to_string(), str_ty);
        assert_eq!(tcx.resolve_alias("X"), Some(str_ty));
    }

    #[test]
    fn test_new_type_var() {
        let mut tcx = TypeContext::new();
        let id = tcx.new_type_var("T".to_string(), None, Vec::new());
        assert_eq!(id, TypeVarId(0));
        let info = tcx.get_type_var(id);
        assert_eq!(info.name, "T");
        assert!(info.bound.is_none());
        assert!(info.constraints.is_empty());
    }

    #[test]
    fn test_new_type_var_with_bound() {
        let mut tcx = TypeContext::new();
        let int_ty = tcx.int();
        let id = tcx.new_type_var("T".to_string(), Some(int_ty), Vec::new());
        let info = tcx.get_type_var(id);
        assert_eq!(info.bound, Some(int_ty));
    }

    #[test]
    fn test_new_type_var_with_constraints() {
        let mut tcx = TypeContext::new();
        let int_ty = tcx.int();
        let str_ty = tcx.str();
        let id = tcx.new_type_var(
            "T".to_string(), None, vec![int_ty, str_ty],
        );
        let info = tcx.get_type_var(id);
        assert_eq!(info.constraints, vec![int_ty, str_ty]);
    }

    #[test]
    fn test_multiple_type_vars() {
        let mut tcx = TypeContext::new();
        let id0 = tcx.new_type_var("T".to_string(), None, Vec::new());
        let id1 = tcx.new_type_var("U".to_string(), None, Vec::new());
        assert_eq!(id0, TypeVarId(0));
        assert_eq!(id1, TypeVarId(1));
        assert_eq!(tcx.get_type_var(id0).name, "T");
        assert_eq!(tcx.get_type_var(id1).name, "U");
    }

    // --- Subtype tests ---

    #[test]
    fn test_subtype_reflexive() {
        let tcx = TypeContext::new();
        assert!(tcx.is_subtype(tcx.int(), tcx.int()));
        assert!(tcx.is_subtype(tcx.str(), tcx.str()));
        assert!(tcx.is_subtype(tcx.none(), tcx.none()));
    }

    #[test]
    fn test_subtype_any_is_universal() {
        let tcx = TypeContext::new();
        // Any as supertype
        assert!(tcx.is_subtype(tcx.int(), tcx.any()));
        assert!(tcx.is_subtype(tcx.str(), tcx.any()));
        assert!(tcx.is_subtype(tcx.none(), tcx.any()));
        // Any as subtype
        assert!(tcx.is_subtype(tcx.any(), tcx.int()));
        assert!(tcx.is_subtype(tcx.any(), tcx.str()));
    }

    #[test]
    fn test_subtype_never_is_bottom() {
        let tcx = TypeContext::new();
        assert!(tcx.is_subtype(tcx.never(), tcx.int()));
        assert!(tcx.is_subtype(tcx.never(), tcx.str()));
        assert!(tcx.is_subtype(tcx.never(), tcx.none()));
        assert!(tcx.is_subtype(tcx.never(), tcx.any()));
    }

    #[test]
    fn test_subtype_int_to_float() {
        let tcx = TypeContext::new();
        assert!(tcx.is_subtype(tcx.int(), tcx.float()));
        assert!(!tcx.is_subtype(tcx.float(), tcx.int()));
    }

    #[test]
    fn test_subtype_bool_to_int() {
        let tcx = TypeContext::new();
        assert!(tcx.is_subtype(tcx.bool(), tcx.int()));
        assert!(!tcx.is_subtype(tcx.int(), tcx.bool()));
    }

    #[test]
    fn test_subtype_bool_chain() {
        let tcx = TypeContext::new();
        // bool -> int -> float
        assert!(tcx.is_subtype(tcx.bool(), tcx.int()));
        assert!(tcx.is_subtype(tcx.int(), tcx.float()));
        // bool -> float (transitive through int widening? No, only direct rules)
        // bool is not directly subtype of float in the current impl
        assert!(!tcx.is_subtype(tcx.bool(), tcx.float()));
    }

    #[test]
    fn test_subtype_no_relation() {
        let tcx = TypeContext::new();
        assert!(!tcx.is_subtype(tcx.str(), tcx.int()));
        assert!(!tcx.is_subtype(tcx.int(), tcx.str()));
        assert!(!tcx.is_subtype(tcx.none(), tcx.int()));
    }

    #[test]
    fn test_subtype_union_supertype() {
        let mut tcx = TypeContext::new();
        let int_ty = tcx.int();
        let str_ty = tcx.str();
        let union_ty = tcx.intern(Ty::Union(vec![int_ty, str_ty]));
        // int is subtype of Union[int, str]
        assert!(tcx.is_subtype(int_ty, union_ty));
        // str is subtype of Union[int, str]
        assert!(tcx.is_subtype(str_ty, union_ty));
        // float is NOT subtype of Union[int, str]
        assert!(!tcx.is_subtype(tcx.float(), union_ty));
    }

    #[test]
    fn test_subtype_union_subtype() {
        let mut tcx = TypeContext::new();
        let int_ty = tcx.int();
        let float_ty = tcx.float();
        // Union[int] is subtype of float because int <: float
        let union_int = tcx.intern(Ty::Union(vec![int_ty]));
        assert!(tcx.is_subtype(union_int, float_ty));
        // Union[int, str] is NOT subtype of float
        let str_ty = tcx.str();
        let union_mixed = tcx.intern(Ty::Union(vec![int_ty, str_ty]));
        assert!(!tcx.is_subtype(union_mixed, float_ty));
    }

    #[test]
    fn test_default_impl() {
        let tcx = TypeContext::default();
        assert_eq!(tcx.int(), TypeId(3));
    }
}

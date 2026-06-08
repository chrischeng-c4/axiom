/// Generics support for Mamba (#314 R1, R3).
///
/// Implements PEP 695 type parameter syntax and generic type resolution.
/// Tracks type variables, bounds, and constraints for generic classes and functions.

use std::collections::HashMap;
use super::ty::{TypeId, TypeVarId, Ty};
use super::context::TypeContext;

/// A type variable with optional bound and constraints.
#[derive(Debug, Clone)]
pub struct TypeVar {
    pub id: TypeVarId,
    pub name: String,
    /// Upper bound: T: SomeType (T must be subtype of bound)
    pub bound: Option<TypeId>,
    /// Constraints: T(int, str) means T must be exactly one of these
    pub constraints: Vec<TypeId>,
}

/// A generic parameter list (e.g., `class Box[T]` or `def f[T, U]`).
#[derive(Debug, Clone)]
pub struct GenericParams {
    pub params: Vec<TypeVar>,
}

impl GenericParams {
    pub fn new() -> Self {
        Self { params: Vec::new() }
    }

    pub fn add(&mut self, name: &str, id: TypeVarId, bound: Option<TypeId>) {
        self.params.push(TypeVar {
            id,
            name: name.to_string(),
            bound,
            constraints: Vec::new(),
        });
    }

    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    pub fn len(&self) -> usize {
        self.params.len()
    }
}

/// Substitution map: TypeVarId → concrete TypeId.
#[derive(Debug, Clone)]
pub struct Substitution {
    map: HashMap<TypeVarId, TypeId>,
}

impl Substitution {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn insert(&mut self, var: TypeVarId, ty: TypeId) {
        self.map.insert(var, ty);
    }

    pub fn get(&self, var: TypeVarId) -> Option<TypeId> {
        self.map.get(&var).copied()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Apply this substitution to a type, replacing type variables with
    /// their concrete types. Requires mutable TypeContext to intern new types.
    pub fn apply(&self, ty: TypeId, tcx: &mut TypeContext) -> TypeId {
        let ty_val = tcx.get(ty).clone();
        match ty_val {
            Ty::TypeVar(var_id) => {
                self.map.get(&var_id).copied().unwrap_or(ty)
            }
            Ty::List(elem) => {
                let new_elem = self.apply(elem, tcx);
                if new_elem == elem { ty } else { tcx.intern(Ty::List(new_elem)) }
            }
            Ty::Dict(k, v) => {
                let new_k = self.apply(k, tcx);
                let new_v = self.apply(v, tcx);
                if new_k == k && new_v == v { ty }
                else { tcx.intern(Ty::Dict(new_k, new_v)) }
            }
            Ty::Tuple(ref elems) => {
                let new_elems: Vec<TypeId> = elems.iter()
                    .map(|e| self.apply(*e, tcx))
                    .collect();
                if new_elems == *elems { ty }
                else { tcx.intern(Ty::Tuple(new_elems)) }
            }
            Ty::Fn { ref params, ret, variadic } => {
                let new_params: Vec<TypeId> = params.iter()
                    .map(|p| self.apply(*p, tcx))
                    .collect();
                let new_ret = self.apply(ret, tcx);
                if new_params == *params && new_ret == ret { ty }
                else { tcx.intern(Ty::Fn { params: new_params, ret: new_ret, variadic }) }
            }
            Ty::Union(ref variants) => {
                let new_variants: Vec<TypeId> = variants.iter()
                    .map(|v| self.apply(*v, tcx))
                    .collect();
                if new_variants == *variants { ty }
                else { tcx.intern(Ty::Union(new_variants)) }
            }
            // Primitive types are unchanged
            _ => ty,
        }
    }
}

/// Infer type arguments by unifying generic parameters with concrete arguments.
///
/// Given `def f[T](x: T, y: T)` called as `f(1, 2)`,
/// this produces {T → int}.
///
/// Returns the substitution and a list of conflict errors.
pub fn infer_type_args(
    generic_params: &GenericParams,
    param_types: &[TypeId],
    arg_types: &[TypeId],
    tcx: &TypeContext,
) -> (Substitution, Vec<String>) {
    let mut subst = Substitution::new();
    let mut conflicts = Vec::new();

    for (param_ty, arg_ty) in param_types.iter().zip(arg_types.iter()) {
        unify_for_inference(
            *param_ty, *arg_ty, &generic_params.params,
            &mut subst, &mut conflicts, tcx,
        );
    }

    (subst, conflicts)
}

/// Attempt to unify a parameter type with an argument type to infer type variables.
fn unify_for_inference(
    param: TypeId,
    arg: TypeId,
    type_vars: &[TypeVar],
    subst: &mut Substitution,
    conflicts: &mut Vec<String>,
    tcx: &TypeContext,
) {
    let param_ty = tcx.get(param).clone();
    match param_ty {
        Ty::TypeVar(var_id) => {
            // Check this is one of our generic type vars
            if type_vars.iter().any(|tv| tv.id == var_id) {
                if let Some(existing) = subst.get(var_id) {
                    // Already inferred — verify consistency
                    if existing != arg {
                        let tv_name = type_vars.iter()
                            .find(|tv| tv.id == var_id)
                            .map(|tv| tv.name.as_str())
                            .unwrap_or("?");
                        conflicts.push(format!(
                            "conflicting types for type parameter '{tv_name}'"
                        ));
                    }
                } else {
                    subst.insert(var_id, arg);
                }
            }
        }
        Ty::List(elem_param) => {
            let arg_ty = tcx.get(arg).clone();
            if let Ty::List(elem_arg) = arg_ty {
                unify_for_inference(elem_param, elem_arg, type_vars, subst, conflicts, tcx);
            }
        }
        Ty::Dict(k_param, v_param) => {
            let arg_ty = tcx.get(arg).clone();
            if let Ty::Dict(k_arg, v_arg) = arg_ty {
                unify_for_inference(k_param, k_arg, type_vars, subst, conflicts, tcx);
                unify_for_inference(v_param, v_arg, type_vars, subst, conflicts, tcx);
            }
        }
        Ty::Tuple(params_inner) => {
            let arg_ty = tcx.get(arg).clone();
            if let Ty::Tuple(args_inner) = arg_ty {
                for (p, a) in params_inner.iter().zip(args_inner.iter()) {
                    unify_for_inference(*p, *a, type_vars, subst, conflicts, tcx);
                }
            }
        }
        _ => {
            // Concrete type — no inference needed
        }
    }
}

/// Check that a substitution satisfies all type variable bounds.
pub fn check_bounds(
    subst: &Substitution,
    generic_params: &GenericParams,
    tcx: &TypeContext,
) -> Vec<String> {
    let mut errors = Vec::new();

    for tv in &generic_params.params {
        if let Some(concrete) = subst.get(tv.id) {
            // Check bound
            if let Some(bound) = tv.bound {
                if !tcx.is_subtype(concrete, bound) {
                    errors.push(format!(
                        "Type parameter '{}' bound violation: expected subtype of {:?}",
                        tv.name, tcx.get(bound)
                    ));
                }
            }
            // Check constraints
            if !tv.constraints.is_empty() && !tv.constraints.contains(&concrete) {
                errors.push(format!(
                    "Type parameter '{}' must be one of the constrained types",
                    tv.name
                ));
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitution_apply() {
        let mut tcx = TypeContext::new();
        let int_ty = tcx.int();
        let var_id = TypeVarId(0);
        let var_ty = tcx.intern(Ty::TypeVar(var_id));

        let mut subst = Substitution::new();
        subst.insert(var_id, int_ty);

        // TypeVar → int
        assert_eq!(subst.apply(var_ty, &mut tcx), int_ty);
        // int → int (unchanged)
        assert_eq!(subst.apply(int_ty, &mut tcx), int_ty);
    }

    #[test]
    fn test_substitution_in_list() {
        let mut tcx = TypeContext::new();
        let int_ty = tcx.int();
        let var_id = TypeVarId(0);
        let var_ty = tcx.intern(Ty::TypeVar(var_id));
        let list_of_var = tcx.intern(Ty::List(var_ty));

        let mut subst = Substitution::new();
        subst.insert(var_id, int_ty);

        let result = subst.apply(list_of_var, &mut tcx);
        assert_eq!(*tcx.get(result), Ty::List(int_ty));
    }

    #[test]
    fn test_infer_type_args() {
        let mut tcx = TypeContext::new();
        let int_ty = tcx.int();
        let var_id = TypeVarId(0);
        let var_ty = tcx.intern(Ty::TypeVar(var_id));

        let mut gp = GenericParams::new();
        gp.add("T", var_id, None);

        let (subst, conflicts) = infer_type_args(&gp, &[var_ty, var_ty], &[int_ty, int_ty], &tcx);
        assert!(conflicts.is_empty());
        assert_eq!(subst.get(var_id), Some(int_ty));
    }

    #[test]
    fn test_generic_params_empty() {
        let gp = GenericParams::new();
        assert!(gp.is_empty());
        assert_eq!(gp.len(), 0);
    }

    #[test]
    fn test_generic_params_add() {
        let mut gp = GenericParams::new();
        gp.add("T", TypeVarId(0), None);
        assert!(!gp.is_empty());
        assert_eq!(gp.len(), 1);
        assert_eq!(gp.params[0].name, "T");
        assert_eq!(gp.params[0].id, TypeVarId(0));
        assert!(gp.params[0].bound.is_none());
    }

    #[test]
    fn test_generic_params_with_bound() {
        let mut gp = GenericParams::new();
        let bound = TypeId(3); // int
        gp.add("T", TypeVarId(0), Some(bound));
        assert_eq!(gp.params[0].bound, Some(bound));
    }

    #[test]
    fn test_substitution_empty() {
        let subst = Substitution::new();
        assert!(subst.is_empty());
        assert_eq!(subst.get(TypeVarId(0)), None);
    }

    #[test]
    fn test_substitution_insert_get() {
        let mut subst = Substitution::new();
        subst.insert(TypeVarId(0), TypeId(3));
        assert!(!subst.is_empty());
        assert_eq!(subst.get(TypeVarId(0)), Some(TypeId(3)));
        assert_eq!(subst.get(TypeVarId(1)), None);
    }

    #[test]
    fn test_substitution_overwrite() {
        let mut subst = Substitution::new();
        subst.insert(TypeVarId(0), TypeId(3));
        subst.insert(TypeVarId(0), TypeId(5));
        assert_eq!(subst.get(TypeVarId(0)), Some(TypeId(5)));
    }

    #[test]
    fn test_substitution_apply_unchanged_primitive() {
        let mut tcx = TypeContext::new();
        let subst = Substitution::new();
        // Applying empty subst to int returns int
        assert_eq!(subst.apply(tcx.int(), &mut tcx), tcx.int());
        assert_eq!(subst.apply(tcx.str(), &mut tcx), tcx.str());
    }

    #[test]
    fn test_substitution_apply_unbound_var() {
        let mut tcx = TypeContext::new();
        let var_id = TypeVarId(99);
        let var_ty = tcx.intern(Ty::TypeVar(var_id));
        let subst = Substitution::new(); // no mapping for var_id
        // Unbound typevar stays unchanged
        assert_eq!(subst.apply(var_ty, &mut tcx), var_ty);
    }

    #[test]
    fn test_substitution_apply_dict() {
        let mut tcx = TypeContext::new();
        let var_id_k = TypeVarId(0);
        let var_id_v = TypeVarId(1);
        let var_k = tcx.intern(Ty::TypeVar(var_id_k));
        let var_v = tcx.intern(Ty::TypeVar(var_id_v));
        let dict_ty = tcx.intern(Ty::Dict(var_k, var_v));

        let int_ty = tcx.int();
        let str_ty = tcx.str();
        let mut subst = Substitution::new();
        subst.insert(var_id_k, str_ty);
        subst.insert(var_id_v, int_ty);

        let result = subst.apply(dict_ty, &mut tcx);
        assert_eq!(*tcx.get(result), Ty::Dict(str_ty, int_ty));
    }

    #[test]
    fn test_substitution_apply_tuple() {
        let mut tcx = TypeContext::new();
        let var_id = TypeVarId(0);
        let var_ty = tcx.intern(Ty::TypeVar(var_id));
        let int_ty = tcx.int();
        let tuple_ty = tcx.intern(Ty::Tuple(vec![var_ty, int_ty]));

        let mut subst = Substitution::new();
        subst.insert(var_id, tcx.str());

        let result = subst.apply(tuple_ty, &mut tcx);
        assert_eq!(*tcx.get(result), Ty::Tuple(vec![tcx.str(), int_ty]));
    }

    #[test]
    fn test_substitution_apply_fn() {
        let mut tcx = TypeContext::new();
        let var_id = TypeVarId(0);
        let var_ty = tcx.intern(Ty::TypeVar(var_id));
        let fn_ty = tcx.intern(Ty::Fn { params: vec![var_ty], ret: var_ty, variadic: false });

        let mut subst = Substitution::new();
        subst.insert(var_id, tcx.int());

        let result = subst.apply(fn_ty, &mut tcx);
        let int_ty = tcx.int();
        assert_eq!(*tcx.get(result), Ty::Fn { params: vec![int_ty], ret: int_ty, variadic: false });
    }

    #[test]
    fn test_substitution_apply_union() {
        let mut tcx = TypeContext::new();
        let var_id = TypeVarId(0);
        let var_ty = tcx.intern(Ty::TypeVar(var_id));
        let union_ty = tcx.intern(Ty::Union(vec![var_ty, tcx.none()]));

        let mut subst = Substitution::new();
        subst.insert(var_id, tcx.int());

        let result = subst.apply(union_ty, &mut tcx);
        let int_ty = tcx.int();
        let none_ty = tcx.none();
        assert_eq!(*tcx.get(result), Ty::Union(vec![int_ty, none_ty]));
    }

    #[test]
    fn test_substitution_apply_no_change_returns_same_id() {
        let mut tcx = TypeContext::new();
        let int_ty = tcx.int();
        let list_int = tcx.intern(Ty::List(int_ty));
        let subst = Substitution::new();
        // No vars to substitute; same TypeId returned
        let result = subst.apply(list_int, &mut tcx);
        assert_eq!(result, list_int);
    }

    #[test]
    fn test_infer_type_args_conflict() {
        let mut tcx = TypeContext::new();
        let int_ty = tcx.int();
        let str_ty = tcx.str();
        let var_id = TypeVarId(0);
        let var_ty = tcx.intern(Ty::TypeVar(var_id));

        let mut gp = GenericParams::new();
        gp.add("T", var_id, None);

        // T is inferred as int from first arg, then str from second → conflict
        let (_, conflicts) = infer_type_args(&gp, &[var_ty, var_ty], &[int_ty, str_ty], &tcx);
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].contains("conflicting types"));
    }

    #[test]
    fn test_infer_type_args_no_matching_var() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();

        let mut gp = GenericParams::new();
        gp.add("T", TypeVarId(0), None);

        // param is concrete int, not a type var — nothing inferred
        let (subst, conflicts) = infer_type_args(&gp, &[int_ty], &[int_ty], &tcx);
        assert!(conflicts.is_empty());
        assert!(subst.is_empty());
    }

    #[test]
    fn test_infer_type_args_through_list() {
        let mut tcx = TypeContext::new();
        let var_id = TypeVarId(0);
        let var_ty = tcx.intern(Ty::TypeVar(var_id));
        let list_var = tcx.intern(Ty::List(var_ty));
        let int_ty = tcx.int();
        let list_int = tcx.intern(Ty::List(int_ty));

        let mut gp = GenericParams::new();
        gp.add("T", var_id, None);

        let (subst, conflicts) = infer_type_args(&gp, &[list_var], &[list_int], &tcx);
        assert!(conflicts.is_empty());
        assert_eq!(subst.get(var_id), Some(int_ty));
    }

    #[test]
    fn test_infer_type_args_through_dict() {
        let mut tcx = TypeContext::new();
        let var_k = TypeVarId(0);
        let var_v = TypeVarId(1);
        let var_k_ty = tcx.intern(Ty::TypeVar(var_k));
        let var_v_ty = tcx.intern(Ty::TypeVar(var_v));
        let dict_var = tcx.intern(Ty::Dict(var_k_ty, var_v_ty));

        let str_ty = tcx.str();
        let int_ty = tcx.int();
        let dict_concrete = tcx.intern(Ty::Dict(str_ty, int_ty));

        let mut gp = GenericParams::new();
        gp.add("K", var_k, None);
        gp.add("V", var_v, None);

        let (subst, conflicts) = infer_type_args(&gp, &[dict_var], &[dict_concrete], &tcx);
        assert!(conflicts.is_empty());
        assert_eq!(subst.get(var_k), Some(str_ty));
        assert_eq!(subst.get(var_v), Some(int_ty));
    }

    #[test]
    fn test_infer_mismatched_structure_no_crash() {
        let mut tcx = TypeContext::new();
        let var_id = TypeVarId(0);
        let var_ty = tcx.intern(Ty::TypeVar(var_id));
        let list_var = tcx.intern(Ty::List(var_ty));
        let int_ty = tcx.int();
        // arg is plain int, not a list — unification should be a no-op
        let mut gp = GenericParams::new();
        gp.add("T", var_id, None);

        let (subst, conflicts) = infer_type_args(&gp, &[list_var], &[int_ty], &tcx);
        assert!(conflicts.is_empty());
        assert!(subst.is_empty()); // nothing inferred
    }

    #[test]
    fn test_check_bounds_passes() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let float_ty = tcx.float();

        let mut gp = GenericParams::new();
        // T: float (T must be subtype of float)
        gp.add("T", TypeVarId(0), Some(float_ty));

        let mut subst = Substitution::new();
        subst.insert(TypeVarId(0), int_ty); // int <: float ✓

        let errors = check_bounds(&subst, &gp, &tcx);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_bounds_violation() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let str_ty = tcx.str();

        let mut gp = GenericParams::new();
        // T: int (T must be subtype of int)
        gp.add("T", TypeVarId(0), Some(int_ty));

        let mut subst = Substitution::new();
        subst.insert(TypeVarId(0), str_ty); // str <: int? No!

        let errors = check_bounds(&subst, &gp, &tcx);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("bound violation"));
    }

    #[test]
    fn test_check_bounds_constraint_satisfied() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let str_ty = tcx.str();

        let mut gp = GenericParams::new();
        let tv = super::TypeVar {
            id: TypeVarId(0),
            name: "T".to_string(),
            bound: None,
            constraints: vec![int_ty, str_ty],
        };
        // Manually add to params
        gp.params.push(tv);

        let mut subst = Substitution::new();
        subst.insert(TypeVarId(0), int_ty); // int is in constraints

        let errors = check_bounds(&subst, &gp, &tcx);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_bounds_constraint_violated() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let str_ty = tcx.str();
        let float_ty = tcx.float();

        let mut gp = GenericParams::new();
        gp.params.push(super::TypeVar {
            id: TypeVarId(0),
            name: "T".to_string(),
            bound: None,
            constraints: vec![int_ty, str_ty],
        });

        let mut subst = Substitution::new();
        subst.insert(TypeVarId(0), float_ty); // float NOT in {int, str}

        let errors = check_bounds(&subst, &gp, &tcx);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("constrained types"));
    }

    #[test]
    fn test_check_bounds_unresolved_var() {
        let tcx = TypeContext::new();
        let float_ty = tcx.float();

        let mut gp = GenericParams::new();
        gp.add("T", TypeVarId(0), Some(float_ty));

        // No entry for TypeVarId(0) in subst → skipped, no error
        let subst = Substitution::new();
        let errors = check_bounds(&subst, &gp, &tcx);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_infer_through_tuple() {
        let mut tcx = TypeContext::new();
        let var_id = TypeVarId(0);
        let var_ty = tcx.intern(Ty::TypeVar(var_id));
        let int_ty = tcx.int();
        let tuple_param = tcx.intern(Ty::Tuple(vec![var_ty, int_ty]));
        let str_ty = tcx.str();
        let tuple_arg = tcx.intern(Ty::Tuple(vec![str_ty, int_ty]));

        let mut gp = GenericParams::new();
        gp.add("T", var_id, None);

        let (subst, conflicts) = infer_type_args(&gp, &[tuple_param], &[tuple_arg], &tcx);
        assert!(conflicts.is_empty());
        assert_eq!(subst.get(var_id), Some(str_ty));
    }
}

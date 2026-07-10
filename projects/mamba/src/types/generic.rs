use super::context::TypeContext;
use super::ty::{Ty, TypeId, TypeParamDefault, TypeVarId, TypeVarKind};
/// Generics support for Mamba (#314 R1, R3).
///
/// Implements PEP 695 type parameter syntax and generic type resolution.
/// Tracks type variables, bounds, and constraints for generic classes and functions.
use std::collections::{HashMap, HashSet};

/// A type variable with optional bound and constraints.
#[derive(Debug, Clone)]
pub struct TypeVar {
    pub id: TypeVarId,
    pub name: String,
    pub kind: TypeVarKind,
    /// Upper bound: T: SomeType (T must be subtype of bound)
    pub bound: Option<TypeId>,
    /// Constraints: T(int, str) means T must be exactly one of these
    pub constraints: Vec<TypeId>,
    /// Resolved PEP 696 default type argument, when statically type-shaped.
    pub default: TypeParamDefault,
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
        self.add_with_constraints(name, id, bound, Vec::new());
    }

    pub fn add_with_constraints(
        &mut self,
        name: &str,
        id: TypeVarId,
        bound: Option<TypeId>,
        constraints: Vec<TypeId>,
    ) {
        self.add_param(
            name,
            id,
            TypeVarKind::TypeVar,
            bound,
            constraints,
            TypeParamDefault::None,
        );
    }

    pub fn add_param(
        &mut self,
        name: &str,
        id: TypeVarId,
        kind: TypeVarKind,
        bound: Option<TypeId>,
        constraints: Vec<TypeId>,
        default: TypeParamDefault,
    ) {
        self.params.push(TypeVar {
            id,
            name: name.to_string(),
            kind,
            bound,
            constraints,
            default,
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
        Self {
            map: HashMap::new(),
        }
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
            Ty::TypeVar(var_id) => self.map.get(&var_id).copied().unwrap_or(ty),
            Ty::AliasRef(alias_id) => {
                let source = tcx.alias_instance(alias_id).clone();
                let new_args: Vec<_> = source
                    .args
                    .iter()
                    .map(|arg| self.apply(*arg, tcx))
                    .collect();
                if new_args == source.args {
                    return ty;
                }

                let (specialized_id, specialized_ty) = tcx.intern_alias_instance(
                    source.symbol,
                    source.name,
                    new_args,
                    source.display_arg_count,
                );
                if let Some(source_target) = source.target {
                    if tcx.begin_alias_target(specialized_id) {
                        let specialized_target = self.apply(source_target, tcx);
                        tcx.set_alias_target(specialized_id, specialized_target);
                    }
                }
                specialized_ty
            }
            Ty::List(elem) => {
                let new_elem = self.apply(elem, tcx);
                if new_elem == elem {
                    ty
                } else {
                    tcx.intern(Ty::List(new_elem))
                }
            }
            Ty::Set(elem) => {
                let new_elem = self.apply(elem, tcx);
                if new_elem == elem {
                    ty
                } else {
                    tcx.intern(Ty::Set(new_elem))
                }
            }
            Ty::Dict(k, v) => {
                let new_k = self.apply(k, tcx);
                let new_v = self.apply(v, tcx);
                if new_k == k && new_v == v {
                    ty
                } else {
                    tcx.intern(Ty::Dict(new_k, new_v))
                }
            }
            Ty::Tuple(ref elems) => {
                let new_elems: Vec<TypeId> = elems.iter().map(|e| self.apply(*e, tcx)).collect();
                if new_elems == *elems {
                    ty
                } else {
                    tcx.intern(Ty::Tuple(new_elems))
                }
            }
            Ty::Fn {
                ref params,
                ret,
                variadic,
            } => {
                let new_params: Vec<TypeId> = params.iter().map(|p| self.apply(*p, tcx)).collect();
                let new_ret = self.apply(ret, tcx);
                if new_params == *params && new_ret == ret {
                    ty
                } else {
                    tcx.intern(Ty::Fn {
                        params: new_params,
                        ret: new_ret,
                        variadic,
                    })
                }
            }
            Ty::Union(ref variants) => {
                let new_variants: Vec<TypeId> =
                    variants.iter().map(|v| self.apply(*v, tcx)).collect();
                if new_variants == *variants {
                    ty
                } else {
                    tcx.intern(Ty::Union(new_variants))
                }
            }
            Ty::Class {
                ref name,
                role,
                ref user,
                ref fields,
                ref match_args,
            } => {
                let new_user = user.as_ref().map(|user| super::ty::UserClass {
                    symbol: user.symbol,
                    args: user.args.iter().map(|arg| self.apply(*arg, tcx)).collect(),
                });
                let new_fields: Vec<_> = fields
                    .iter()
                    .map(|(field_name, field_ty)| (field_name.clone(), self.apply(*field_ty, tcx)))
                    .collect();
                if new_user == *user && new_fields == *fields {
                    ty
                } else {
                    tcx.intern(Ty::Class {
                        name: name.clone(),
                        role,
                        user: new_user,
                        fields: new_fields,
                        match_args: match_args.clone(),
                    })
                }
            }
            // Primitive types are unchanged
            _ => ty,
        }
    }
}

/// Bind explicit type arguments to a declaration's type parameters.
///
/// Ordinary TypeVars have fixed arity and may consume trailing defaults.
/// TypeVarTuple and ParamSpec still require richer pack/parameter-list types;
/// keep their legacy positional binding without claiming fixed-arity support.
pub fn bind_explicit_type_args(
    generic_params: &GenericParams,
    supplied: &[TypeId],
    tcx: &mut TypeContext,
) -> (Substitution, Vec<TypeId>, Vec<String>) {
    let mut subst = Substitution::new();
    let mut resolved = supplied.to_vec();

    if generic_params
        .params
        .iter()
        .any(|param| param.kind != TypeVarKind::TypeVar)
    {
        for (param, concrete) in generic_params.params.iter().zip(supplied) {
            subst.insert(param.id, *concrete);
        }
        let errors = check_bounds(&subst, generic_params, tcx);
        return (subst, resolved, errors);
    }

    let total = generic_params.params.len();
    let required = generic_params
        .params
        .iter()
        .filter(|param| !param.default.is_present())
        .count();
    let mut errors = Vec::new();
    if supplied.len() < required || supplied.len() > total {
        let expected = if required == total {
            total.to_string()
        } else {
            format!("between {required} and {total}")
        };
        errors.push(format!(
            "expected {expected} type arguments, got {}",
            supplied.len()
        ));
    }

    for (index, (param, concrete)) in generic_params.params.iter().zip(supplied).enumerate() {
        let concrete = normalize_constrained_candidate(param, *concrete, tcx);
        subst.insert(param.id, concrete);
        resolved[index] = concrete;
    }
    resolved.truncate(total);

    for param in generic_params.params.iter().skip(supplied.len()) {
        let concrete = param
            .default
            .resolved()
            .map(|default| subst.apply(default, tcx))
            .unwrap_or_else(|| tcx.any());
        subst.insert(param.id, concrete);
        resolved.push(concrete);
    }

    errors.extend(check_bounds(&subst, generic_params, tcx));
    (subst, resolved, errors)
}

/// Complete a partially inferred ordinary-TypeVar substitution.
///
/// Constructor inference may solve only some class parameters. Remaining
/// parameters consume their declared default, or `Any` when no default is
/// available. Pack and ParamSpec completion waits for richer representations.
pub fn complete_type_args(
    generic_params: &GenericParams,
    mut subst: Substitution,
    tcx: &mut TypeContext,
) -> Option<(Substitution, Vec<TypeId>)> {
    if generic_params
        .params
        .iter()
        .any(|param| param.kind != TypeVarKind::TypeVar)
    {
        return None;
    }

    let mut resolved = Vec::with_capacity(generic_params.len());
    for param in &generic_params.params {
        let concrete = subst.get(param.id).unwrap_or_else(|| {
            param
                .default
                .resolved()
                .map(|default| subst.apply(default, tcx))
                .unwrap_or_else(|| tcx.any())
        });
        let concrete = normalize_constrained_candidate(param, concrete, tcx);
        subst.insert(param.id, concrete);
        resolved.push(concrete);
    }
    Some((subst, resolved))
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
            *param_ty,
            *arg_ty,
            &generic_params.params,
            &mut subst,
            &mut conflicts,
            tcx,
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
    let mut visiting = HashSet::new();
    unify_for_inference_inner(param, arg, type_vars, subst, conflicts, tcx, &mut visiting);
}

fn unify_for_inference_inner(
    param: TypeId,
    arg: TypeId,
    type_vars: &[TypeVar],
    subst: &mut Substitution,
    conflicts: &mut Vec<String>,
    tcx: &TypeContext,
    visiting: &mut HashSet<(TypeId, TypeId)>,
) {
    if !visiting.insert((param, arg)) {
        return;
    }
    unify_for_inference_step(param, arg, type_vars, subst, conflicts, tcx, visiting);
    visiting.remove(&(param, arg));
}

fn unify_for_inference_step(
    param: TypeId,
    arg: TypeId,
    type_vars: &[TypeVar],
    subst: &mut Substitution,
    conflicts: &mut Vec<String>,
    tcx: &TypeContext,
    visiting: &mut HashSet<(TypeId, TypeId)>,
) {
    let param_ty = tcx.get(param).clone();
    match param_ty {
        Ty::TypeVar(var_id) => {
            // Check this is one of our generic type vars
            if let Some(type_var) = type_vars.iter().find(|tv| tv.id == var_id) {
                let arg = normalize_constrained_candidate(type_var, arg, tcx);
                if let Some(existing) = subst.get(var_id) {
                    // Already inferred — verify consistency
                    if matches!(tcx.get(existing), Ty::Any | Ty::Error)
                        && !matches!(tcx.get(arg), Ty::Any | Ty::Error)
                    {
                        subst.insert(var_id, arg);
                    } else if existing != arg && !matches!(tcx.get(arg), Ty::Any | Ty::Error) {
                        let tv_name = type_var.name.as_str();
                        conflicts.push(format!("conflicting types for type parameter '{tv_name}'"));
                    }
                } else {
                    subst.insert(var_id, arg);
                }
            }
        }
        Ty::AliasRef(param_id) => {
            let param_instance = tcx.alias_instance(param_id).clone();
            let Ty::AliasRef(arg_id) = tcx.get(arg) else {
                if let Some(target) = param_instance.target {
                    unify_for_inference_inner(
                        target, arg, type_vars, subst, conflicts, tcx, visiting,
                    );
                }
                return;
            };
            let arg_instance = tcx.alias_instance(*arg_id).clone();
            if param_instance.symbol == arg_instance.symbol
                && param_instance.args.len() == arg_instance.args.len()
            {
                for (param_arg, arg_arg) in param_instance.args.iter().zip(&arg_instance.args) {
                    unify_for_inference_inner(
                        *param_arg, *arg_arg, type_vars, subst, conflicts, tcx, visiting,
                    );
                }
            } else if let (Some(param_target), Some(arg_target)) =
                (param_instance.target, arg_instance.target)
            {
                unify_for_inference_inner(
                    param_target,
                    arg_target,
                    type_vars,
                    subst,
                    conflicts,
                    tcx,
                    visiting,
                );
            }
        }
        Ty::Union(param_members) => {
            let arg_ty = tcx.get(arg).clone();
            if let Ty::Union(arg_members) = arg_ty {
                let mut used = HashSet::new();
                let mut param_order: Vec<_> = (0..param_members.len()).collect();
                param_order
                    .sort_by_key(|index| matches!(tcx.get(param_members[*index]), Ty::TypeVar(_)));
                for param_index in param_order {
                    let param_member = param_members[param_index];
                    let Some((arg_index, arg_member)) = arg_members
                        .iter()
                        .copied()
                        .enumerate()
                        .find(|(index, arg_member)| {
                            !used.contains(index)
                                && inference_shapes_match(param_member, *arg_member, tcx)
                        })
                    else {
                        continue;
                    };
                    used.insert(arg_index);
                    unify_for_inference_inner(
                        param_member,
                        arg_member,
                        type_vars,
                        subst,
                        conflicts,
                        tcx,
                        visiting,
                    );
                }
            } else if let Some(param_member) = param_members
                .iter()
                .copied()
                .find(|param_member| inference_shapes_match(*param_member, arg, tcx))
            {
                unify_for_inference_inner(
                    param_member,
                    arg,
                    type_vars,
                    subst,
                    conflicts,
                    tcx,
                    visiting,
                );
            }
        }
        Ty::List(elem_param) => {
            let arg_ty = tcx.get(arg).clone();
            if let Ty::List(elem_arg) = arg_ty {
                unify_for_inference_inner(
                    elem_param, elem_arg, type_vars, subst, conflicts, tcx, visiting,
                );
            }
        }
        Ty::Set(elem_param) => {
            let arg_ty = tcx.get(arg).clone();
            if let Ty::Set(elem_arg) = arg_ty {
                unify_for_inference_inner(
                    elem_param, elem_arg, type_vars, subst, conflicts, tcx, visiting,
                );
            }
        }
        Ty::Dict(k_param, v_param) => {
            let arg_ty = tcx.get(arg).clone();
            if let Ty::Dict(k_arg, v_arg) = arg_ty {
                unify_for_inference_inner(
                    k_param, k_arg, type_vars, subst, conflicts, tcx, visiting,
                );
                unify_for_inference_inner(
                    v_param, v_arg, type_vars, subst, conflicts, tcx, visiting,
                );
            }
        }
        Ty::Tuple(params_inner) => {
            let arg_ty = tcx.get(arg).clone();
            if let Ty::Tuple(args_inner) = arg_ty {
                for (p, a) in params_inner.iter().zip(args_inner.iter()) {
                    unify_for_inference_inner(*p, *a, type_vars, subst, conflicts, tcx, visiting);
                }
            }
        }
        Ty::Class {
            role: param_role,
            user: Some(param_user),
            ..
        } => {
            let arg_ty = tcx.get(arg).clone();
            if let Ty::Class {
                role: arg_role,
                user: Some(arg_user),
                ..
            } = arg_ty
            {
                if param_user.symbol == arg_user.symbol
                    && param_role == arg_role
                    && param_user.args.len() == arg_user.args.len()
                {
                    for (param_arg, concrete_arg) in param_user.args.iter().zip(&arg_user.args) {
                        unify_for_inference_inner(
                            *param_arg,
                            *concrete_arg,
                            type_vars,
                            subst,
                            conflicts,
                            tcx,
                            visiting,
                        );
                    }
                }
            }
        }
        _ => {
            // Concrete type — no inference needed
        }
    }
}

fn inference_shapes_match(param: TypeId, arg: TypeId, tcx: &TypeContext) -> bool {
    if param == arg {
        return true;
    }
    match (tcx.get(param), tcx.get(arg)) {
        (Ty::TypeVar(_), _) | (Ty::Any | Ty::Error, _) | (_, Ty::Any | Ty::Error) => true,
        (Ty::AliasRef(_), Ty::AliasRef(_))
        | (Ty::List(_), Ty::List(_))
        | (Ty::Set(_), Ty::Set(_))
        | (Ty::Dict(_, _), Ty::Dict(_, _))
        | (Ty::Tuple(_), Ty::Tuple(_))
        | (Ty::Union(_), Ty::Union(_))
        | (Ty::Fn { .. }, Ty::Fn { .. }) => true,
        (
            Ty::Class {
                name: param_name,
                role: param_role,
                user: param_user,
                ..
            },
            Ty::Class {
                name: arg_name,
                role: arg_role,
                user: arg_user,
                ..
            },
        ) => {
            param_role == arg_role
                && match (param_user, arg_user) {
                    (Some(param), Some(arg)) => param.symbol == arg.symbol,
                    (None, None) => param_name == arg_name,
                    _ => false,
                }
        }
        (left, right) => std::mem::discriminant(left) == std::mem::discriminant(right),
    }
}

fn normalize_constrained_candidate(type_var: &TypeVar, arg: TypeId, tcx: &TypeContext) -> TypeId {
    if type_var.constraints.is_empty() || matches!(tcx.get(arg), Ty::Any | Ty::Error) {
        return arg;
    }
    if type_var.constraints.contains(&arg) {
        return arg;
    }

    let mut matches = type_var
        .constraints
        .iter()
        .copied()
        .filter(|constraint| tcx.is_subtype(arg, *constraint));
    let Some(mut best) = matches.next() else {
        return arg;
    };
    for candidate in matches {
        if tcx.is_subtype(candidate, best) && !tcx.is_subtype(best, candidate) {
            best = candidate;
        }
    }
    best
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
                        tv.name,
                        tcx.get(bound)
                    ));
                }
            }
            // Check constraints
            let constraint_matches = tv
                .constraints
                .iter()
                .any(|constraint| tcx.is_subtype(concrete, *constraint));
            if !tv.constraints.is_empty() && !constraint_matches {
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
    fn substitution_specializes_recursive_alias_instances() {
        let mut tcx = TypeContext::new();
        let var = tcx.new_type_var("T".to_string(), None, Vec::new());
        let var_ty = tcx.intern(Ty::TypeVar(var));
        let symbol = crate::resolve::SymbolId(17);
        let (generic_instance, generic_ref) =
            tcx.intern_alias_instance(symbol, "Chain".to_string(), vec![var_ty], 1);
        let generic_target = tcx.intern(Ty::List(generic_ref));
        tcx.set_alias_target(generic_instance, generic_target);

        let mut subst = Substitution::new();
        subst.insert(var, tcx.int());
        let specialized_ref = subst.apply(generic_ref, &mut tcx);
        let Ty::AliasRef(specialized_instance) = tcx.get(specialized_ref) else {
            panic!("recursive alias substitution lost its identity");
        };
        let specialized_instance = *specialized_instance;
        assert_eq!(
            tcx.alias_instance(specialized_instance).args,
            vec![tcx.int()]
        );
        let specialized_target = tcx
            .alias_target(specialized_instance)
            .expect("specialized recursive alias target was not backfilled");
        let Ty::List(nested) = tcx.get(specialized_target) else {
            panic!("specialized target lost its productive list head");
        };
        assert_eq!(*nested, specialized_ref);
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
    fn test_infer_type_args_refines_gradual_evidence() {
        let mut tcx = TypeContext::new();
        let var_id = TypeVarId(0);
        let var_ty = tcx.intern(Ty::TypeVar(var_id));
        let mut gp = GenericParams::new();
        gp.add("T", var_id, None);

        let (subst, conflicts) =
            infer_type_args(&gp, &[var_ty, var_ty], &[tcx.any(), tcx.int()], &tcx);
        assert!(conflicts.is_empty());
        assert_eq!(subst.get(var_id), Some(tcx.int()));

        let (subst, conflicts) =
            infer_type_args(&gp, &[var_ty, var_ty], &[tcx.int(), tcx.any()], &tcx);
        assert!(conflicts.is_empty());
        assert_eq!(subst.get(var_id), Some(tcx.int()));
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
    fn test_bind_explicit_type_args_enforces_fixed_arity() {
        let mut tcx = TypeContext::new();
        let mut gp = GenericParams::new();
        gp.add("T", TypeVarId(0), None);
        gp.add("U", TypeVarId(1), None);

        let int_ty = tcx.int();
        let str_ty = tcx.str();
        let bool_ty = tcx.bool();
        let (_, _, errors) = bind_explicit_type_args(&gp, &[int_ty], &mut tcx);
        assert_eq!(errors, vec!["expected 2 type arguments, got 1"]);

        let (_, _, errors) = bind_explicit_type_args(&gp, &[int_ty, str_ty, bool_ty], &mut tcx);
        assert_eq!(errors, vec!["expected 2 type arguments, got 3"]);
    }

    #[test]
    fn test_bind_explicit_type_args_applies_trailing_defaults() {
        let mut tcx = TypeContext::new();
        let t_id = TypeVarId(0);
        let u_id = TypeVarId(1);
        let t_ty = tcx.intern(Ty::TypeVar(t_id));
        let default_u = tcx.intern(Ty::List(t_ty));
        let mut gp = GenericParams::new();
        gp.add_param(
            "T",
            t_id,
            TypeVarKind::TypeVar,
            None,
            Vec::new(),
            TypeParamDefault::None,
        );
        gp.add_param(
            "U",
            u_id,
            TypeVarKind::TypeVar,
            None,
            Vec::new(),
            TypeParamDefault::Resolved(default_u),
        );

        let str_ty = tcx.str();
        let (subst, resolved, errors) = bind_explicit_type_args(&gp, &[str_ty], &mut tcx);
        assert!(errors.is_empty());
        assert_eq!(resolved.len(), 2);
        assert_eq!(subst.get(t_id), Some(str_ty));
        assert_eq!(*tcx.get(subst.get(u_id).unwrap()), Ty::List(str_ty));
    }

    #[test]
    fn test_bind_explicit_type_args_keeps_variadics_skip_safe() {
        let mut tcx = TypeContext::new();
        let mut gp = GenericParams::new();
        gp.add_param(
            "Ts",
            TypeVarId(0),
            TypeVarKind::TypeVarTuple,
            None,
            Vec::new(),
            TypeParamDefault::None,
        );

        let (_, resolved, errors) = bind_explicit_type_args(&gp, &[tcx.int(), tcx.str()], &mut tcx);
        assert!(errors.is_empty());
        assert_eq!(resolved.len(), 2);
    }

    #[test]
    fn test_bind_explicit_type_args_promotes_constrained_subtypes() {
        let mut tcx = TypeContext::new();
        let mut gp = GenericParams::new();
        gp.add_with_constraints("T", TypeVarId(0), None, vec![tcx.int(), tcx.str()]);

        let (subst, resolved, errors) = bind_explicit_type_args(&gp, &[tcx.bool()], &mut tcx);
        assert!(errors.is_empty());
        assert_eq!(subst.get(TypeVarId(0)), Some(tcx.int()));
        assert_eq!(resolved, vec![tcx.int()]);
    }

    #[test]
    fn test_complete_type_args_combines_inference_defaults_and_any() {
        let mut tcx = TypeContext::new();
        let t_id = TypeVarId(0);
        let u_id = TypeVarId(1);
        let v_id = TypeVarId(2);
        let t_ty = tcx.intern(Ty::TypeVar(t_id));
        let default_u = tcx.intern(Ty::List(t_ty));
        let mut gp = GenericParams::new();
        gp.add("T", t_id, None);
        gp.add_param(
            "U",
            u_id,
            TypeVarKind::TypeVar,
            None,
            Vec::new(),
            TypeParamDefault::Resolved(default_u),
        );
        gp.add("V", v_id, None);

        let mut inferred = Substitution::new();
        inferred.insert(t_id, tcx.str());
        let (completed, resolved) = complete_type_args(&gp, inferred, &mut tcx).unwrap();

        assert_eq!(completed.get(t_id), Some(tcx.str()));
        assert_eq!(*tcx.get(completed.get(u_id).unwrap()), Ty::List(tcx.str()));
        assert_eq!(completed.get(v_id), Some(tcx.any()));
        assert_eq!(resolved.len(), 3);
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
        let fn_ty = tcx.intern(Ty::Fn {
            params: vec![var_ty],
            ret: var_ty,
            variadic: false,
        });

        let mut subst = Substitution::new();
        subst.insert(var_id, tcx.int());

        let result = subst.apply(fn_ty, &mut tcx);
        let int_ty = tcx.int();
        assert_eq!(
            *tcx.get(result),
            Ty::Fn {
                params: vec![int_ty],
                ret: int_ty,
                variadic: false
            }
        );
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
    fn test_substitution_apply_user_class_identity_and_fields() {
        let mut tcx = TypeContext::new();
        let var_id = TypeVarId(0);
        let var_ty = tcx.intern(Ty::TypeVar(var_id));
        let class_ty = tcx.intern(Ty::Class {
            name: "Box".to_string(),
            role: crate::types::ty::ClassRole::Instance,
            user: Some(crate::types::ty::UserClass {
                symbol: crate::resolve::SymbolId(42),
                args: vec![var_ty],
            }),
            fields: vec![("value".to_string(), var_ty)],
            match_args: None,
        });
        let mut subst = Substitution::new();
        subst.insert(var_id, tcx.int());

        let applied = subst.apply(class_ty, &mut tcx);
        let Ty::Class {
            name,
            user: Some(user),
            fields,
            ..
        } = tcx.get(applied)
        else {
            panic!("expected specialized user class");
        };
        assert_eq!(name, "Box");
        assert_eq!(user.symbol, crate::resolve::SymbolId(42));
        assert!(matches!(
            tcx.get(applied),
            Ty::Class {
                role: crate::types::ty::ClassRole::Instance,
                ..
            }
        ));
        assert_eq!(user.args, vec![tcx.int()]);
        assert_eq!(fields, &vec![("value".to_string(), tcx.int())]);
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
    fn test_constrained_inference_promotes_and_prefers_exact_match() {
        let mut tcx = TypeContext::new();
        let var_id = TypeVarId(0);
        let var_ty = tcx.intern(Ty::TypeVar(var_id));
        let bool_ty = tcx.bool();
        let int_ty = tcx.int();
        let float_ty = tcx.float();

        let mut gp = GenericParams::new();
        gp.add_with_constraints("T", var_id, None, vec![float_ty, int_ty]);

        let (bool_subst, conflicts) = infer_type_args(&gp, &[var_ty], &[bool_ty], &tcx);
        assert!(conflicts.is_empty());
        assert_eq!(bool_subst.get(var_id), Some(int_ty));

        let (int_subst, conflicts) = infer_type_args(&gp, &[var_ty], &[int_ty], &tcx);
        assert!(conflicts.is_empty());
        assert_eq!(int_subst.get(var_id), Some(int_ty));
    }

    #[test]
    fn test_constrained_inference_compares_promoted_candidates() {
        let mut tcx = TypeContext::new();
        let var_id = TypeVarId(0);
        let var_ty = tcx.intern(Ty::TypeVar(var_id));
        let bool_ty = tcx.bool();
        let int_ty = tcx.int();

        let mut gp = GenericParams::new();
        gp.add_with_constraints("T", var_id, None, vec![int_ty, tcx.str()]);

        let (subst, conflicts) = infer_type_args(&gp, &[var_ty, var_ty], &[bool_ty, int_ty], &tcx);
        assert!(conflicts.is_empty());
        assert_eq!(subst.get(var_id), Some(int_ty));
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
            kind: TypeVarKind::TypeVar,
            bound: None,
            constraints: vec![int_ty, str_ty],
            default: TypeParamDefault::None,
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
            kind: TypeVarKind::TypeVar,
            bound: None,
            constraints: vec![int_ty, str_ty],
            default: TypeParamDefault::None,
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

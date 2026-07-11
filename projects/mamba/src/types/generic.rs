use super::context::{AliasIdentity, TypeContext};
use super::ty::{
    CallableParam, CallableParamKind, ParamPack, ParamPackTail, Ty, TypeId, TypePack,
    TypeParamDefault, TypeVarId, TypeVarKind,
};
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

/// Scalar type and callable-parameter-pack substitutions.
#[derive(Debug, Clone)]
pub struct Substitution {
    map: HashMap<TypeVarId, TypeId>,
    param_packs: HashMap<TypeVarId, ParamPack>,
    type_packs: HashMap<TypeVarId, TypePack>,
}

impl Substitution {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            param_packs: HashMap::new(),
            type_packs: HashMap::new(),
        }
    }

    pub fn insert(&mut self, var: TypeVarId, ty: TypeId) {
        self.map.insert(var, ty);
    }

    pub fn get(&self, var: TypeVarId) -> Option<TypeId> {
        self.map.get(&var).copied()
    }

    pub(crate) fn scalar_bindings(&self) -> Vec<(TypeVarId, TypeId)> {
        let mut bindings: Vec<_> = self.map.iter().map(|(var, ty)| (*var, *ty)).collect();
        bindings.sort_by_key(|(var, _)| var.0);
        bindings
    }

    pub(crate) fn param_pack_bindings(&self) -> Vec<(TypeVarId, ParamPack)> {
        let mut bindings: Vec<_> = self
            .param_packs
            .iter()
            .map(|(var, pack)| (*var, pack.clone()))
            .collect();
        bindings.sort_by_key(|(var, _)| var.0);
        bindings
    }

    pub(crate) fn type_pack_bindings(&self) -> Vec<(TypeVarId, TypePack)> {
        let mut bindings: Vec<_> = self
            .type_packs
            .iter()
            .map(|(var, pack)| (*var, pack.clone()))
            .collect();
        bindings.sort_by_key(|(var, _)| var.0);
        bindings
    }

    pub(crate) fn from_bindings(
        scalar: &[(TypeVarId, TypeId)],
        param_packs: &[(TypeVarId, ParamPack)],
        type_packs: &[(TypeVarId, TypePack)],
    ) -> Self {
        let mut substitution = Self::new();
        for (var, ty) in scalar {
            substitution.insert(*var, *ty);
        }
        for (var, pack) in param_packs {
            substitution.insert_param_pack(*var, pack.clone());
        }
        for (var, pack) in type_packs {
            substitution.insert_type_pack(*var, pack.clone());
        }
        substitution
    }

    pub fn insert_param_pack(&mut self, var: TypeVarId, pack: ParamPack) {
        self.param_packs.insert(var, pack);
    }

    pub fn get_param_pack(&self, var: TypeVarId) -> Option<&ParamPack> {
        self.param_packs.get(&var)
    }

    pub fn insert_type_pack(&mut self, var: TypeVarId, pack: TypePack) {
        self.type_packs.insert(var, pack);
    }

    pub fn get_type_pack(&self, var: TypeVarId) -> Option<&TypePack> {
        self.type_packs.get(&var)
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty() && self.param_packs.is_empty() && self.type_packs.is_empty()
    }

    pub fn apply_param_pack(&self, pack: &ParamPack, tcx: &mut TypeContext) -> ParamPack {
        self.apply_param_pack_inner(pack, tcx, &mut HashSet::new())
    }

    fn apply_param_pack_inner(
        &self,
        pack: &ParamPack,
        tcx: &mut TypeContext,
        visiting: &mut HashSet<TypeVarId>,
    ) -> ParamPack {
        let mut params: Vec<_> = pack
            .params
            .iter()
            .map(|param| CallableParam {
                name: param.name.clone(),
                ty: self.apply_inner(param.ty, tcx, visiting),
                kind: param.kind,
                has_default: param.has_default,
            })
            .collect();
        let tail = match pack.tail {
            ParamPackTail::ParamSpec(var) if visiting.insert(var) => {
                if let Some(bound) = self.param_packs.get(&var) {
                    let bound = self.apply_param_pack_inner(bound, tcx, visiting);
                    params.extend(bound.params);
                    visiting.remove(&var);
                    bound.tail
                } else {
                    visiting.remove(&var);
                    ParamPackTail::ParamSpec(var)
                }
            }
            tail => tail,
        };
        ParamPack { params, tail }
    }

    /// Apply this substitution to a type, replacing type variables with
    /// their concrete types. Requires mutable TypeContext to intern new types.
    pub fn apply(&self, ty: TypeId, tcx: &mut TypeContext) -> TypeId {
        self.apply_inner(ty, tcx, &mut HashSet::new())
    }

    fn apply_type_list_inner(
        &self,
        items: &[TypeId],
        tcx: &mut TypeContext,
        visiting_packs: &mut HashSet<TypeVarId>,
    ) -> Vec<TypeId> {
        let mut applied = Vec::new();
        for item in items {
            if let Ty::Unpack(var) = tcx.get(*item) {
                let var = *var;
                if let Some(pack) = self.type_packs.get(&var) {
                    if visiting_packs.insert(var) {
                        applied.extend(self.apply_type_list_inner(
                            &pack.types,
                            tcx,
                            visiting_packs,
                        ));
                        visiting_packs.remove(&var);
                        continue;
                    }
                }
            }
            applied.push(self.apply_inner(*item, tcx, visiting_packs));
        }
        applied
    }

    fn apply_inner(
        &self,
        ty: TypeId,
        tcx: &mut TypeContext,
        visiting_param_packs: &mut HashSet<TypeVarId>,
    ) -> TypeId {
        let ty_val = tcx.get(ty).clone();
        match ty_val {
            Ty::TypeVar(var_id) => self.map.get(&var_id).copied().unwrap_or(ty),
            Ty::AliasRef(alias_id) => {
                let source = tcx.alias_instance(alias_id).clone();
                let mut new_args = self.apply_type_list_inner(
                    &source.args[..source.display_arg_count],
                    tcx,
                    visiting_param_packs,
                );
                let new_display_arg_count = new_args.len();
                new_args.extend(self.apply_type_list_inner(
                    &source.args[source.display_arg_count..],
                    tcx,
                    visiting_param_packs,
                ));
                if new_args == source.args
                    && new_display_arg_count == source.display_arg_count
                {
                    return ty;
                }

                let (specialized_id, specialized_ty) = tcx.intern_alias_instance(
                    source.identity,
                    source.name,
                    new_args,
                    new_display_arg_count,
                );
                if tcx.alias_target_is_rejected(alias_id) {
                    tcx.reject_alias_target(specialized_id);
                } else if let Some(source_target) = source.target {
                    if tcx.begin_alias_target(specialized_id) {
                        let specialized_target =
                            self.apply_inner(source_target, tcx, visiting_param_packs);
                        if tcx.alias_has_unguarded_cycle(specialized_id, specialized_target)
                            || tcx.alias_target_has_invalid_generated_edge(
                                specialized_id,
                                specialized_target,
                            )
                        {
                            tcx.reject_alias_target(specialized_id);
                        } else {
                            tcx.set_alias_target(specialized_id, specialized_target);
                        }
                    }
                } else if matches!(source.identity, AliasIdentity::Generated(_, _))
                    && tcx.alias_target(specialized_id).is_none()
                    && !tcx.alias_target_is_resolving(specialized_id)
                {
                    tcx.defer_alias_target(
                        specialized_id,
                        alias_id,
                        self.scalar_bindings(),
                        self.param_pack_bindings(),
                        self.type_pack_bindings(),
                    );
                }
                specialized_ty
            }
            Ty::List(elem) => {
                let new_elem = self.apply_inner(elem, tcx, visiting_param_packs);
                if new_elem == elem {
                    ty
                } else {
                    tcx.intern(Ty::List(new_elem))
                }
            }
            Ty::Set(elem) => {
                let new_elem = self.apply_inner(elem, tcx, visiting_param_packs);
                if new_elem == elem {
                    ty
                } else {
                    tcx.intern(Ty::Set(new_elem))
                }
            }
            Ty::TypeObject(instance) => {
                let new_instance = self.apply_inner(instance, tcx, visiting_param_packs);
                if new_instance == instance {
                    ty
                } else {
                    tcx.intern(Ty::TypeObject(new_instance))
                }
            }
            Ty::Dict(k, v) => {
                let new_k = self.apply_inner(k, tcx, visiting_param_packs);
                let new_v = self.apply_inner(v, tcx, visiting_param_packs);
                if new_k == k && new_v == v {
                    ty
                } else {
                    tcx.intern(Ty::Dict(new_k, new_v))
                }
            }
            Ty::Tuple(ref elems) => {
                let new_elems =
                    self.apply_type_list_inner(elems, tcx, visiting_param_packs);
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
                ref signature,
                param_spec,
            } => {
                let bound_var = param_spec.filter(|var| {
                    self.param_packs.contains_key(var) && visiting_param_packs.insert(*var)
                });
                let new_params =
                    self.apply_type_list_inner(params, tcx, visiting_param_packs);
                let new_ret = self.apply_inner(ret, tcx, visiting_param_packs);
                let new_signature = signature.as_ref().map(|params| {
                    params
                        .iter()
                        .map(|param| CallableParam {
                            name: param.name.clone(),
                            ty: self.apply_inner(param.ty, tcx, visiting_param_packs),
                            kind: param.kind,
                            has_default: param.has_default,
                        })
                        .collect::<Vec<_>>()
                });
                let bound_pack = bound_var
                    .and_then(|var| self.param_packs.get(&var))
                    .map(|pack| self.apply_param_pack_inner(pack, tcx, visiting_param_packs));
                let (new_params, new_signature, new_variadic, new_param_spec) =
                    if let Some(pack) = bound_pack {
                        let mut full_signature = new_signature.unwrap_or_else(|| {
                            new_params
                                .iter()
                                .map(|ty| CallableParam {
                                    name: None,
                                    ty: *ty,
                                    kind: CallableParamKind::PosOnly,
                                    has_default: false,
                                })
                                .collect()
                        });
                        full_signature.extend(pack.params);
                        let compact = full_signature
                            .iter()
                            .take_while(|param| param.kind != CallableParamKind::VarPos)
                            .filter(|param| {
                                matches!(
                                    param.kind,
                                    CallableParamKind::PosOnly
                                        | CallableParamKind::PosOrKw
                                        | CallableParamKind::KwOnly
                                )
                            })
                            .map(|param| param.ty)
                            .collect();
                        let (variadic, param_spec) = match pack.tail {
                            ParamPackTail::Closed => (
                                full_signature.iter().any(|param| {
                                    matches!(
                                        param.kind,
                                        CallableParamKind::VarPos | CallableParamKind::VarKw
                                    )
                                }),
                                None,
                            ),
                            ParamPackTail::Ellipsis => (true, None),
                            ParamPackTail::ParamSpec(var) => (false, Some(var)),
                        };
                        (compact, Some(full_signature), variadic, param_spec)
                    } else {
                        (new_params, new_signature, variadic, param_spec)
                    };
                let result = if new_params == *params
                    && new_ret == ret
                    && new_signature == *signature
                    && new_variadic == variadic
                    && new_param_spec == param_spec
                {
                    ty
                } else {
                    tcx.intern(Ty::Fn {
                        params: new_params,
                        ret: new_ret,
                        variadic: new_variadic,
                        signature: new_signature,
                        param_spec: new_param_spec,
                    })
                };
                if let Some(var) = bound_var {
                    visiting_param_packs.remove(&var);
                }
                result
            }
            Ty::Union(ref variants) => {
                let new_variants: Vec<TypeId> = variants
                    .iter()
                    .map(|v| self.apply_inner(*v, tcx, visiting_param_packs))
                    .collect();
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
                ref external,
                ref fields,
                ref match_args,
            } => {
                let new_user = user.as_ref().map(|user| super::ty::UserClass {
                    symbol: user.symbol,
                    args: self.apply_type_list_inner(
                        &user.args,
                        tcx,
                        visiting_param_packs,
                    ),
                });
                let new_external = external.as_ref().map(|external| super::ty::ExternalClass {
                    module: external.module.clone(),
                    name: external.name.clone(),
                    args: self.apply_type_list_inner(
                        &external.args,
                        tcx,
                        visiting_param_packs,
                    ),
                });
                let new_fields: Vec<_> = fields
                    .iter()
                    .map(|(field_name, field_ty)| {
                        (
                            field_name.clone(),
                            self.apply_inner(*field_ty, tcx, visiting_param_packs),
                        )
                    })
                    .collect();
                if new_user == *user && new_external == *external && new_fields == *fields {
                    ty
                } else {
                    tcx.intern(Ty::Class {
                        name: name.clone(),
                        role,
                        user: new_user,
                        external: new_external,
                        fields: new_fields,
                        match_args: match_args.clone(),
                    })
                }
            }
            Ty::External(super::ty::ExternalValue::Callable(ref callable)) => {
                let mut new_callable = callable.clone();
                if let Some(receiver) = &callable.receiver {
                    new_callable.receiver = Some(super::ty::ExternalClass {
                        module: receiver.module.clone(),
                        name: receiver.name.clone(),
                        args: receiver
                            .args
                            .iter()
                            .map(|arg| self.apply_inner(*arg, tcx, visiting_param_packs))
                            .collect(),
                    });
                }
                if new_callable == *callable {
                    ty
                } else {
                    tcx.intern(Ty::External(super::ty::ExternalValue::Callable(
                        new_callable,
                    )))
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
/// One TypeVarTuple consumes the ordered middle slice between fixed prefix and
/// suffix parameters. ParamSpec and unsupported mixed-pack declarations retain
/// their legacy skip-safe positional behavior.
pub fn bind_explicit_type_args(
    generic_params: &GenericParams,
    supplied: &[TypeId],
    tcx: &mut TypeContext,
) -> (Substitution, Vec<TypeId>, Vec<String>) {
    let (subst, resolved, mut errors) =
        bind_explicit_type_args_without_bounds(generic_params, supplied, tcx);
    errors.extend(check_bounds(&subst, generic_params, tcx));
    (subst, resolved, errors)
}

/// Bind explicit type arguments without choosing a relation for bounds.
/// External generated contracts use their protocol-aware three-state relation;
/// user declarations keep using `bind_explicit_type_args` above.
pub fn bind_explicit_type_args_without_bounds(
    generic_params: &GenericParams,
    supplied: &[TypeId],
    tcx: &mut TypeContext,
) -> (Substitution, Vec<TypeId>, Vec<String>) {
    let mut subst = Substitution::new();
    let mut resolved = supplied.to_vec();

    if let Some(pack_index) = generic_params
        .params
        .iter()
        .position(|param| param.kind == TypeVarKind::TypeVarTuple)
        .filter(|_| {
            generic_params
                .params
                .iter()
                .filter(|param| param.kind == TypeVarKind::TypeVarTuple)
                .count()
                == 1
                && generic_params
                    .params
                    .iter()
                    .all(|param| param.kind != TypeVarKind::ParamSpec)
        })
    {
        let trailing = generic_params.params.len() - pack_index - 1;
        let required = generic_params
            .params
            .iter()
            .filter(|param| {
                param.kind != TypeVarKind::TypeVarTuple && !param.default.is_present()
            })
            .count();
        let mut errors = Vec::new();
        if supplied.len() < required {
            errors.push(format!(
                "expected at least {required} type arguments, got {}",
                supplied.len()
            ));
        }
        resolved.clear();
        for (index, param) in generic_params.params[..pack_index].iter().enumerate() {
            let concrete = supplied
                .get(index)
                .copied()
                .or_else(|| {
                    param
                        .default
                        .resolved()
                        .map(|default| subst.apply(default, tcx))
                })
                .unwrap_or_else(|| tcx.any());
            let concrete = normalize_scalar_explicit_arg(param, concrete, tcx, &mut errors);
            subst.insert(param.id, concrete);
            resolved.push(concrete);
        }
        let pack_start = pack_index.min(supplied.len());
        let pack_end = supplied
            .len()
            .saturating_sub(trailing)
            .max(pack_start);
        let pack = TypePack {
            types: supplied[pack_start..pack_end].to_vec(),
        };
        resolved.extend(pack.types.iter().copied());
        subst.insert_type_pack(
            generic_params.params[pack_index].id,
            pack,
        );
        for (offset, param) in generic_params.params[pack_index + 1..].iter().enumerate() {
            let index = pack_end + offset;
            let concrete = supplied
                .get(index)
                .copied()
                .or_else(|| {
                    param
                        .default
                        .resolved()
                        .map(|default| subst.apply(default, tcx))
                })
                .unwrap_or_else(|| tcx.any());
            let concrete = normalize_scalar_explicit_arg(param, concrete, tcx, &mut errors);
            subst.insert(param.id, concrete);
            resolved.push(concrete);
        }
        return (subst, resolved, errors);
    }

    if generic_params
        .params
        .iter()
        .any(|param| param.kind != TypeVarKind::TypeVar)
    {
        let mut errors = Vec::new();
        for (index, (param, concrete)) in generic_params.params.iter().zip(supplied).enumerate() {
            let concrete = normalize_scalar_explicit_arg(param, *concrete, tcx, &mut errors);
            subst.insert(param.id, concrete);
            resolved[index] = concrete;
        }
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
        let concrete = normalize_scalar_explicit_arg(param, *concrete, tcx, &mut errors);
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

    (subst, resolved, errors)
}

fn normalize_scalar_explicit_arg(
    param: &TypeVar,
    candidate: TypeId,
    tcx: &TypeContext,
    errors: &mut Vec<String>,
) -> TypeId {
    if matches!(tcx.get(candidate), Ty::Unpack(_)) {
        errors.push(format!(
            "type parameter '{}' does not accept an unpacked type argument",
            param.name
        ));
        return tcx.error();
    }
    normalize_constrained_candidate(param, candidate, tcx)
}

/// Complete a partially inferred ordinary-TypeVar substitution.
///
/// Constructor inference may solve only some class parameters. Remaining
/// parameters consume their declared default, or `Any` when no default is
/// available. Inferred pack parameters require a pack-aware completion path;
/// this scalar completion helper deliberately declines them.
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

/// Complete type arguments for a callable consumer that understands ParamPack.
pub fn complete_callable_type_args(
    generic_params: &GenericParams,
    mut subst: Substitution,
    tcx: &mut TypeContext,
) -> Option<Substitution> {
    for param in &generic_params.params {
        match param.kind {
            TypeVarKind::ParamSpec => {
                subst.get_param_pack(param.id)?;
            }
            TypeVarKind::TypeVarTuple => return None,
            TypeVarKind::TypeVar => {
                let concrete = subst.get(param.id).unwrap_or_else(|| {
                    param
                        .default
                        .resolved()
                        .map(|default| subst.apply(default, tcx))
                        .unwrap_or_else(|| tcx.any())
                });
                let concrete = normalize_constrained_candidate(param, concrete, tcx);
                subst.insert(param.id, concrete);
            }
        }
    }
    Some(subst)
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

fn callable_param_pack(
    params: Vec<TypeId>,
    variadic: bool,
    signature: Option<Vec<CallableParam>>,
    param_spec: Option<TypeVarId>,
) -> ParamPack {
    let has_signature = signature.is_some();
    let params = signature.unwrap_or_else(|| {
        params
            .into_iter()
            .map(|ty| CallableParam {
                name: None,
                ty,
                kind: CallableParamKind::PosOrKw,
                has_default: false,
            })
            .collect()
    });
    let tail = if let Some(var) = param_spec {
        ParamPackTail::ParamSpec(var)
    } else if variadic && !has_signature {
        ParamPackTail::Ellipsis
    } else {
        ParamPackTail::Closed
    };
    ParamPack { params, tail }
}

fn callable_prefix_definitely_incompatible(
    expected: TypeId,
    actual_parameter: TypeId,
    tcx: &TypeContext,
) -> bool {
    if tcx.is_subtype(expected, actual_parameter)
        || tcx.contains_type_var(expected)
        || tcx.contains_type_var(actual_parameter)
    {
        return false;
    }
    let indeterminate = |ty: &Ty| {
        matches!(
            ty,
            Ty::Any
                | Ty::Error
                | Ty::SelfType
                | Ty::Infer(_)
                | Ty::AliasRef(_)
                | Ty::External(_)
                | Ty::Class { user: Some(_), .. }
        )
    };
    !indeterminate(tcx.get(expected)) && !indeterminate(tcx.get(actual_parameter))
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
            if param_instance.identity == arg_instance.identity
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
        Ty::TypeObject(instance_param) => {
            let arg_ty = tcx.get(arg).clone();
            if let Ty::TypeObject(instance_arg) = arg_ty {
                unify_for_inference_inner(
                    instance_param,
                    instance_arg,
                    type_vars,
                    subst,
                    conflicts,
                    tcx,
                    visiting,
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
        Ty::Fn {
            params,
            ret,
            variadic,
            signature,
            param_spec,
        } => {
            let Ty::Fn {
                params: arg_params,
                ret: arg_ret,
                variadic: arg_variadic,
                signature: arg_signature,
                param_spec: arg_param_spec,
            } = tcx.get(arg).clone()
            else {
                return;
            };
            unify_for_inference_inner(ret, arg_ret, type_vars, subst, conflicts, tcx, visiting);
            let expected = callable_param_pack(params, variadic, signature, param_spec);
            if expected.tail == ParamPackTail::Ellipsis {
                return;
            }
            let actual =
                callable_param_pack(arg_params, arg_variadic, arg_signature, arg_param_spec);
            let captures_tail = matches!(expected.tail, ParamPackTail::ParamSpec(_));
            let residual_start = if captures_tail {
                let mut actual_index = 0usize;
                for expected_param in &expected.params {
                    let Some(actual_param) = actual.params.get(actual_index) else {
                        if actual.tail == ParamPackTail::Closed {
                            conflicts.push(
                                "argument type mismatch: callable does not accept the Concatenate prefix positionally"
                                    .to_string(),
                            );
                        }
                        return;
                    };
                    match actual_param.kind {
                        CallableParamKind::PosOnly | CallableParamKind::PosOrKw => {
                            actual_index += 1;
                        }
                        CallableParamKind::VarPos => {
                            if callable_prefix_definitely_incompatible(
                                expected_param.ty,
                                actual_param.ty,
                                tcx,
                            ) {
                                conflicts.push(
                                    "argument type mismatch: callable variadic parameter does not accept the Concatenate prefix"
                                        .to_string(),
                                );
                                return;
                            }
                        }
                        CallableParamKind::KwOnly | CallableParamKind::VarKw => {
                            if actual.tail == ParamPackTail::Closed {
                                conflicts.push(
                                    "argument type mismatch: callable parameter kind does not accept the Concatenate prefix positionally"
                                        .to_string(),
                                );
                            }
                            return;
                        }
                    }
                    unify_for_inference_inner(
                        expected_param.ty,
                        actual_param.ty,
                        type_vars,
                        subst,
                        conflicts,
                        tcx,
                        visiting,
                    );
                }
                actual_index
            } else {
                if actual.params.len() < expected.params.len() {
                    return;
                }
                for (expected_param, actual_param) in expected.params.iter().zip(&actual.params) {
                    unify_for_inference_inner(
                        expected_param.ty,
                        actual_param.ty,
                        type_vars,
                        subst,
                        conflicts,
                        tcx,
                        visiting,
                    );
                }
                return;
            };
            let ParamPackTail::ParamSpec(var) = expected.tail else {
                return;
            };
            if !type_vars
                .iter()
                .any(|param| param.id == var && param.kind == TypeVarKind::ParamSpec)
            {
                return;
            }
            let residual = ParamPack {
                params: actual.params[residual_start..].to_vec(),
                tail: actual.tail,
            };
            if let Some(existing) = subst.get_param_pack(var) {
                if existing != &residual {
                    let name = type_vars
                        .iter()
                        .find(|param| param.id == var)
                        .map(|param| param.name.as_str())
                        .unwrap_or("P");
                    conflicts.push(format!(
                        "conflicting callable parameter packs for type parameter '{name}'"
                    ));
                }
            } else {
                subst.insert_param_pack(var, residual);
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
        Ty::Class {
            role: param_role,
            external: Some(param_external),
            ..
        } => {
            let arg_ty = tcx.get(arg).clone();
            if let Ty::Class {
                role: arg_role,
                external: Some(arg_external),
                ..
            } = &arg_ty
            {
                if param_external.module == arg_external.module
                    && param_external.name == arg_external.name
                    && param_role == *arg_role
                    && param_external.args.len() == arg_external.args.len()
                {
                    for (param_arg, concrete_arg) in
                        param_external.args.iter().zip(&arg_external.args)
                    {
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
                    return;
                }
            }
            if param_role != super::ty::ClassRole::Instance || param_external.module != "typing" {
                return;
            }
            let projected = match (param_external.name.as_str(), arg_ty) {
                ("Iterable" | "Collection" | "Sequence" | "MutableSequence", Ty::List(item))
                | ("Iterable" | "Collection", Ty::Set(item)) => Some(vec![item]),
                ("Iterable" | "Collection", Ty::Dict(key, _)) => Some(vec![key]),
                ("Mapping" | "MutableMapping", Ty::Dict(key, value)) => Some(vec![key, value]),
                ("Iterable" | "Collection" | "Sequence", Ty::Str) => Some(vec![tcx.str()]),
                (
                    "Iterable" | "Collection" | "Sequence" | "MutableSequence",
                    Ty::Class {
                        external: Some(actual),
                        ..
                    },
                ) if actual.module == "builtins" && actual.name == "bytearray" => {
                    Some(vec![tcx.int()])
                }
                _ => None,
            };
            if let Some(projected) = projected {
                if projected.len() == param_external.args.len() {
                    for (param_arg, concrete_arg) in param_external.args.iter().zip(projected) {
                        unify_for_inference_inner(
                            *param_arg,
                            concrete_arg,
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
        | (Ty::TypeObject(_), Ty::TypeObject(_))
        | (Ty::Dict(_, _), Ty::Dict(_, _))
        | (Ty::Tuple(_), Ty::Tuple(_))
        | (Ty::Union(_), Ty::Union(_))
        | (Ty::Fn { .. }, Ty::Fn { .. }) => true,
        (
            Ty::Class {
                name: param_name,
                role: param_role,
                user: param_user,
                external: param_external,
                ..
            },
            Ty::Class {
                name: arg_name,
                role: arg_role,
                user: arg_user,
                external: arg_external,
                ..
            },
        ) => {
            param_role == arg_role
                && match (param_user, arg_user, param_external, arg_external) {
                    (Some(param), Some(arg), None, None) => param.symbol == arg.symbol,
                    (None, None, Some(param), Some(arg)) => {
                        param.module == arg.module && param.name == arg.name
                    }
                    (None, None, None, None) => param_name == arg_name,
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
        let (generic_instance, generic_ref) = tcx.intern_alias_instance(
            crate::types::context::AliasIdentity::Source(symbol),
            "Chain".to_string(),
            vec![var_ty],
            1,
        );
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
    fn substitution_defers_transformed_alias_until_template_target_exists() {
        let mut tcx = TypeContext::new();
        let var = tcx.new_type_var("T".to_string(), None, Vec::new());
        let other = tcx.new_type_var("U".to_string(), None, Vec::new());
        let param_spec = tcx.new_type_param(
            "P".to_string(),
            TypeVarKind::ParamSpec,
            None,
            Vec::new(),
            TypeParamDefault::None,
        );
        let type_var_tuple = tcx.new_type_param(
            "Ts".to_string(),
            TypeVarKind::TypeVarTuple,
            None,
            Vec::new(),
            TypeParamDefault::None,
        );
        let var_ty = tcx.intern(Ty::TypeVar(var));
        let other_ty = tcx.intern(Ty::TypeVar(other));
        let (template, template_ref) = tcx.intern_alias_instance(
            crate::types::context::AliasIdentity::Generated(
                crate::types::stdlib_typespec::StrSpecId(1),
                crate::types::stdlib_typespec::StrSpecId(2),
            ),
            "example.A".to_string(),
            vec![var_ty, other_ty],
            2,
        );
        assert!(tcx.begin_alias_target(template));

        let mut subst = Substitution::new();
        subst.insert(other, tcx.str());
        subst.insert(var, tcx.int());
        let pack = ParamPack {
            params: vec![CallableParam {
                name: Some("value".to_string()),
                ty: tcx.int(),
                kind: CallableParamKind::PosOrKw,
                has_default: false,
            }],
            tail: ParamPackTail::Closed,
        };
        subst.insert_param_pack(param_spec, pack.clone());
        let type_pack = TypePack {
            types: vec![tcx.str(), tcx.bool()],
        };
        subst.insert_type_pack(type_var_tuple, type_pack.clone());
        let specialized_ref = subst.apply(template_ref, &mut tcx);
        let Ty::AliasRef(specialized) = tcx.get(specialized_ref) else {
            panic!("transformed specialization lost its alias identity");
        };
        let deferred = tcx
            .deferred_alias_target(*specialized)
            .expect("resolving template must leave a deferred recipe")
            .clone();
        assert_eq!(deferred.template, template);
        assert_eq!(
            deferred.substitutions,
            vec![(var, tcx.int()), (other, tcx.str())]
        );
        assert_eq!(deferred.param_packs, vec![(param_spec, pack.clone())]);
        assert_eq!(
            deferred.type_packs,
            vec![(type_var_tuple, type_pack.clone())]
        );
        let rebuilt = Substitution::from_bindings(
            &deferred.substitutions,
            &deferred.param_packs,
            &deferred.type_packs,
        );
        assert_eq!(rebuilt.get(var), Some(tcx.int()));
        assert_eq!(rebuilt.get(other), Some(tcx.str()));
        assert_eq!(rebuilt.get_param_pack(param_spec), Some(&pack));
        assert_eq!(rebuilt.get_type_pack(type_var_tuple), Some(&type_pack));
        assert_eq!(tcx.alias_target(*specialized), None);

        let target = tcx.intern(Ty::List(template_ref));
        tcx.set_alias_target(template, target);
    }

    #[test]
    fn substitution_rejects_resolved_unguarded_alias_specialization() {
        let mut tcx = TypeContext::new();
        let var = tcx.new_type_var("T".to_string(), None, Vec::new());
        let var_ty = tcx.intern(Ty::TypeVar(var));
        let (template, template_ref) = tcx.intern_alias_instance(
            crate::types::context::AliasIdentity::Generated(
                crate::types::stdlib_typespec::StrSpecId(7),
                crate::types::stdlib_typespec::StrSpecId(8),
            ),
            "example.Direct".to_string(),
            vec![var_ty],
            1,
        );
        tcx.set_alias_target(template, template_ref);

        let mut subst = Substitution::new();
        subst.insert(var, tcx.int());
        let specialized_ref = subst.apply(template_ref, &mut tcx);
        let Ty::AliasRef(specialized) = tcx.get(specialized_ref) else {
            panic!("unguarded specialization lost its alias identity");
        };
        assert_eq!(tcx.alias_target(*specialized), Some(tcx.never()));
        assert!(tcx.alias_target_is_rejected(*specialized));
        assert!(tcx.deferred_alias_target(*specialized).is_none());
    }

    #[test]
    fn substitution_preserves_rejected_generated_alias_identity() {
        let mut tcx = TypeContext::new();
        let var = tcx.new_type_var("T".to_string(), None, Vec::new());
        let var_ty = tcx.intern(Ty::TypeVar(var));
        let (template, template_ref) = tcx.intern_alias_instance(
            crate::types::context::AliasIdentity::Generated(
                crate::types::stdlib_typespec::StrSpecId(9),
                crate::types::stdlib_typespec::StrSpecId(10),
            ),
            "example.Rejected".to_string(),
            vec![var_ty],
            1,
        );
        tcx.reject_alias_target(template);

        let mut subst = Substitution::new();
        subst.insert(var, tcx.int());
        let specialized_ref = subst.apply(template_ref, &mut tcx);
        let Ty::AliasRef(specialized) = tcx.get(specialized_ref) else {
            panic!("rejected specialization lost its alias identity");
        };
        assert_eq!(tcx.alias_target(*specialized), Some(tcx.never()));
        assert!(tcx.alias_target_is_rejected(*specialized));
        assert!(tcx.deferred_alias_target(*specialized).is_none());
        assert_eq!(
            tcx.semantic_head_id(specialized_ref),
            Err(crate::types::context::AliasHeadError::Rejected(
                *specialized
            ))
        );
        assert!(!tcx.is_subtype(specialized_ref, specialized_ref));
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
    fn paramspec_inference_retains_full_callable_pack_and_splices_it() {
        let mut tcx = TypeContext::new();
        let p_id = TypeVarId(40);
        let r_id = TypeVarId(41);
        let r_ty = tcx.intern(Ty::TypeVar(r_id));
        let expected = tcx.intern(Ty::Fn {
            params: Vec::new(),
            ret: r_ty,
            variadic: false,
            signature: None,
            param_spec: Some(p_id),
        });
        let signature = vec![
            CallableParam {
                name: Some("x".to_string()),
                ty: tcx.int(),
                kind: CallableParamKind::PosOnly,
                has_default: false,
            },
            CallableParam {
                name: Some("flag".to_string()),
                ty: tcx.bool(),
                kind: CallableParamKind::KwOnly,
                has_default: true,
            },
            CallableParam {
                name: Some("rest".to_string()),
                ty: tcx.str(),
                kind: CallableParamKind::VarPos,
                has_default: false,
            },
        ];
        let actual = tcx.intern(Ty::Fn {
            params: vec![tcx.int()],
            ret: tcx.str(),
            variadic: true,
            signature: Some(signature.clone()),
            param_spec: None,
        });
        let mut params = GenericParams::new();
        params.add_param(
            "P",
            p_id,
            TypeVarKind::ParamSpec,
            None,
            Vec::new(),
            TypeParamDefault::None,
        );
        params.add("R", r_id, None);

        let (subst, conflicts) = infer_type_args(&params, &[expected], &[actual], &tcx);
        assert!(conflicts.is_empty());
        assert_eq!(subst.get(r_id), Some(tcx.str()));
        assert_eq!(
            subst.get_param_pack(p_id),
            Some(&ParamPack {
                params: signature.clone(),
                tail: ParamPackTail::Closed,
            })
        );
        assert!(
            complete_type_args(&params, subst.clone(), &mut tcx).is_none(),
            "scalar class/alias completion must not collapse ParamSpec to Any"
        );
        let completed = complete_callable_type_args(&params, subst, &mut tcx).unwrap();
        let applied = completed.apply(expected, &mut tcx);
        let Ty::Fn {
            ret,
            signature: Some(applied_signature),
            param_spec,
            ..
        } = tcx.get(applied)
        else {
            panic!("bound ParamSpec must produce a concrete callable signature")
        };
        assert_eq!(*ret, tcx.str());
        assert_eq!(applied_signature, &signature);
        assert_eq!(*param_spec, None);
    }

    #[test]
    fn paramspec_inference_reports_repeated_pack_conflicts() {
        let mut tcx = TypeContext::new();
        let p_id = TypeVarId(50);
        let expected = tcx.intern(Ty::Fn {
            params: Vec::new(),
            ret: tcx.any(),
            variadic: false,
            signature: None,
            param_spec: Some(p_id),
        });
        let one = tcx.intern(Ty::Fn {
            params: vec![tcx.int()],
            ret: tcx.any(),
            variadic: false,
            signature: None,
            param_spec: None,
        });
        let two = tcx.intern(Ty::Fn {
            params: vec![tcx.str()],
            ret: tcx.any(),
            variadic: false,
            signature: None,
            param_spec: None,
        });
        let mut params = GenericParams::new();
        params.add_param(
            "P",
            p_id,
            TypeVarKind::ParamSpec,
            None,
            Vec::new(),
            TypeParamDefault::None,
        );

        let (_, conflicts) = infer_type_args(&params, &[expected, expected], &[one, two], &tcx);
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].contains("conflicting callable parameter packs"));
    }

    #[test]
    fn paramspec_substitution_preserves_cyclic_open_tail_without_duplication() {
        let mut tcx = TypeContext::new();
        let p_id = TypeVarId(60);
        let q_id = TypeVarId(61);
        let callable = tcx.intern(Ty::Fn {
            params: Vec::new(),
            ret: tcx.none(),
            variadic: false,
            signature: None,
            param_spec: Some(p_id),
        });
        let mut subst = Substitution::new();
        subst.insert_param_pack(
            p_id,
            ParamPack {
                params: vec![CallableParam {
                    name: Some("left".to_string()),
                    ty: tcx.int(),
                    kind: CallableParamKind::PosOrKw,
                    has_default: false,
                }],
                tail: ParamPackTail::ParamSpec(q_id),
            },
        );
        subst.insert_param_pack(
            q_id,
            ParamPack {
                params: vec![CallableParam {
                    name: Some("right".to_string()),
                    ty: tcx.str(),
                    kind: CallableParamKind::PosOrKw,
                    has_default: false,
                }],
                tail: ParamPackTail::ParamSpec(p_id),
            },
        );

        let applied = subst.apply(callable, &mut tcx);
        let Ty::Fn {
            signature: Some(signature),
            param_spec,
            ..
        } = tcx.get(applied)
        else {
            panic!("cyclic pack must remain an open callable")
        };
        assert_eq!(signature.len(), 2);
        assert_eq!(signature[0].name.as_deref(), Some("left"));
        assert_eq!(signature[1].name.as_deref(), Some("right"));
        assert_eq!(*param_spec, Some(p_id));
    }

    #[test]
    fn paramspec_substitution_preserves_nested_callable_cycle() {
        let mut tcx = TypeContext::new();
        let p_id = TypeVarId(62);
        let nested = tcx.intern(Ty::Fn {
            params: Vec::new(),
            ret: tcx.none(),
            variadic: false,
            signature: None,
            param_spec: Some(p_id),
        });
        let outer = tcx.intern(Ty::Fn {
            params: Vec::new(),
            ret: tcx.none(),
            variadic: false,
            signature: None,
            param_spec: Some(p_id),
        });
        let mut subst = Substitution::new();
        subst.insert_param_pack(
            p_id,
            ParamPack {
                params: vec![CallableParam {
                    name: Some("callback".to_string()),
                    ty: nested,
                    kind: CallableParamKind::PosOrKw,
                    has_default: false,
                }],
                tail: ParamPackTail::Closed,
            },
        );

        let applied = subst.apply(outer, &mut tcx);
        let Ty::Fn {
            signature: Some(signature),
            param_spec: None,
            ..
        } = tcx.get(applied)
        else {
            panic!("the outer ParamSpec must bind without recursing forever")
        };
        let Ty::Fn {
            param_spec: Some(nested_param_spec),
            ..
        } = tcx.get(signature[0].ty)
        else {
            panic!("the nested cyclic callable must remain open")
        };
        assert_eq!(*nested_param_spec, p_id);
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
            signature: None,
            param_spec: None,
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
                variadic: false,
                signature: None,
                param_spec: None,
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
            external: None,
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

    #[test]
    fn bind_explicit_type_args_keeps_ordered_type_var_tuple_slice() {
        let mut tcx = TypeContext::new();
        let mut gp = GenericParams::new();
        gp.add("Head", TypeVarId(0), None);
        gp.add_param(
            "Ts",
            TypeVarId(1),
            TypeVarKind::TypeVarTuple,
            None,
            Vec::new(),
            TypeParamDefault::None,
        );
        gp.add("Tail", TypeVarId(2), None);

        let int = tcx.int();
        let str_ = tcx.str();
        let bool_ = tcx.bool();
        let (subst, resolved, errors) =
            bind_explicit_type_args(&gp, &[int, str_, bool_], &mut tcx);
        assert!(errors.is_empty());
        assert_eq!(resolved, vec![int, str_, bool_]);
        assert_eq!(subst.get(TypeVarId(0)), Some(int));
        assert_eq!(subst.get(TypeVarId(2)), Some(bool_));
        assert_eq!(
            subst.get_type_pack(TypeVarId(1)).unwrap().types,
            vec![str_]
        );

        let (zero, zero_resolved, zero_errors) =
            bind_explicit_type_args(&gp, &[int, bool_], &mut tcx);
        assert!(zero_errors.is_empty());
        assert_eq!(zero_resolved, vec![int, bool_]);
        assert_eq!(zero.get(TypeVarId(0)), Some(int));
        assert_eq!(zero.get(TypeVarId(2)), Some(bool_));
        assert!(zero.get_type_pack(TypeVarId(1)).unwrap().types.is_empty());

        let float = tcx.float();
        let (many, many_resolved, many_errors) =
            bind_explicit_type_args(&gp, &[int, str_, float, bool_], &mut tcx);
        assert!(many_errors.is_empty());
        assert_eq!(many_resolved, vec![int, str_, float, bool_]);
        assert_eq!(many.get(TypeVarId(0)), Some(int));
        assert_eq!(many.get(TypeVarId(2)), Some(bool_));
        assert_eq!(many.get_type_pack(TypeVarId(1)).unwrap().types, vec![str_, float]);

        let head = tcx.intern(Ty::TypeVar(TypeVarId(0)));
        let unpack = tcx.intern(Ty::Unpack(TypeVarId(1)));
        let tail = tcx.intern(Ty::TypeVar(TypeVarId(2)));
        let template = tcx.intern(Ty::Tuple(vec![head, unpack, tail]));
        let zero_applied = zero.apply(template, &mut tcx);
        assert_eq!(tcx.get(zero_applied), &Ty::Tuple(vec![int, bool_]));
        let one_applied = subst.apply(template, &mut tcx);
        assert_eq!(tcx.get(one_applied), &Ty::Tuple(vec![int, str_, bool_]));
        let many_applied = many.apply(template, &mut tcx);
        assert_eq!(
            tcx.get(many_applied),
            &Ty::Tuple(vec![int, str_, float, bool_])
        );
        let callable = tcx.intern(Ty::Fn {
            params: vec![head, unpack, tail],
            ret: bool_,
            variadic: false,
            signature: None,
            param_spec: None,
        });
        let callable_applied = many.apply(callable, &mut tcx);
        let Ty::Fn { params, ret, .. } = tcx.get(callable_applied) else {
            panic!("TypeVarTuple substitution lost the callable type");
        };
        assert_eq!(params, &vec![int, str_, float, bool_]);
        assert_eq!(*ret, bool_);

        for supplied in [&[][..], &[int][..]] {
            let (under_arity, _, under_arity_errors) =
                bind_explicit_type_args(&gp, supplied, &mut tcx);
            assert_eq!(under_arity_errors.len(), 1);
            assert!(under_arity_errors[0].contains("expected at least 2"));
            assert!(under_arity
                .get_type_pack(TypeVarId(1))
                .unwrap()
                .types
                .is_empty());
        }
    }

    #[test]
    fn bind_explicit_type_args_applies_defaults_around_type_var_tuple() {
        let mut tcx = TypeContext::new();
        let mut gp = GenericParams::new();
        gp.add_param(
            "Head",
            TypeVarId(0),
            TypeVarKind::TypeVar,
            None,
            Vec::new(),
            TypeParamDefault::Resolved(tcx.int()),
        );
        gp.add_param(
            "Ts",
            TypeVarId(1),
            TypeVarKind::TypeVarTuple,
            None,
            Vec::new(),
            TypeParamDefault::None,
        );
        gp.add_param(
            "Tail",
            TypeVarId(2),
            TypeVarKind::TypeVar,
            None,
            Vec::new(),
            TypeParamDefault::Resolved(tcx.str()),
        );

        let (subst, resolved, errors) = bind_explicit_type_args(&gp, &[], &mut tcx);
        assert!(errors.is_empty());
        assert_eq!(resolved, vec![tcx.int(), tcx.str()]);
        assert_eq!(subst.get(TypeVarId(0)), Some(tcx.int()));
        assert_eq!(subst.get(TypeVarId(2)), Some(tcx.str()));
        assert!(subst.get_type_pack(TypeVarId(1)).unwrap().types.is_empty());
    }

    #[test]
    fn bind_explicit_type_args_preserves_paramspec_fallback_without_false_arity() {
        let mut tcx = TypeContext::new();
        let mut gp = GenericParams::new();
        gp.add_param(
            "P",
            TypeVarId(0),
            TypeVarKind::ParamSpec,
            None,
            Vec::new(),
            TypeParamDefault::None,
        );

        let int = tcx.int();
        let str_ = tcx.str();
        let (subst, resolved, errors) =
            bind_explicit_type_args(&gp, &[int, str_], &mut tcx);
        assert!(errors.is_empty());
        assert_eq!(resolved, vec![int, str_]);
        assert_eq!(subst.get(TypeVarId(0)), Some(int));
    }

    #[test]
    fn bind_explicit_type_args_rejects_unpack_for_scalar_parameter() {
        let mut tcx = TypeContext::new();
        let mut gp = GenericParams::new();
        gp.add("T", TypeVarId(0), None);
        let unpack = tcx.intern(Ty::Unpack(TypeVarId(1)));

        let (subst, resolved, errors) = bind_explicit_type_args(&gp, &[unpack], &mut tcx);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("does not accept an unpacked type argument"));
        assert_eq!(resolved, vec![tcx.error()]);
        assert_eq!(subst.get(TypeVarId(0)), Some(tcx.error()));
    }
}

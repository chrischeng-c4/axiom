use super::stdlib_typespec::StrSpecId;
use super::ty::{
    AliasInstanceId, ExternalValue, ParamPack, Ty, TypeId, TypeParamDefault, TypeVarId,
    TypeVarKind,
};
use crate::resolve::SymbolId;
use std::collections::{HashMap, HashSet};

/// Type variable info: optional upper bound and type constraints (#242).
#[derive(Debug, Clone)]
pub struct TypeVarInfo {
    pub name: String,
    pub kind: TypeVarKind,
    pub bound: Option<TypeId>,
    pub constraints: Vec<TypeId>,
    pub default: TypeParamDefault,
}

/// Stable declaration identity shared by source and generated aliases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AliasIdentity {
    Source(SymbolId),
    Generated(StrSpecId, StrSpecId),
}

/// One concrete source or generated alias expansion.
#[derive(Debug, Clone)]
pub struct AliasInstance {
    pub identity: AliasIdentity,
    pub name: String,
    pub args: Vec<TypeId>,
    pub display_arg_count: usize,
    pub target: Option<TypeId>,
    deferred_target: Option<DeferredAliasTarget>,
    rejected: bool,
    resolving: bool,
    ty: TypeId,
}

/// A generated specialization whose template was still resolving when the
/// specialization was first encountered. The template's completed target is
/// substituted lazily before this instance is exposed to a semantic consumer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredAliasTarget {
    pub template: AliasInstanceId,
    pub substitutions: Vec<(TypeVarId, TypeId)>,
    pub param_packs: Vec<(TypeVarId, ParamPack)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AliasHeadError {
    Unresolved(AliasInstanceId),
    Cycle(AliasInstanceId),
    Rejected(AliasInstanceId),
}

/// Interner and registry for all types used during compilation.
#[derive(Debug, Clone)]
pub struct TypeContext {
    types: Vec<Ty>,
    /// Type alias registry: name → resolved TypeId (#241).
    type_aliases: HashMap<String, TypeId>,
    /// Type variable info registry (#242).
    type_vars: Vec<TypeVarInfo>,
    /// Stable recursive-alias nodes, keyed by declaration identity + arguments.
    alias_instances: Vec<AliasInstance>,
    alias_instance_ids: HashMap<(AliasIdentity, Vec<TypeId>), AliasInstanceId>,
    alias_target_undo: Vec<(
        AliasInstanceId,
        Option<TypeId>,
        Option<DeferredAliasTarget>,
        bool,
        bool,
    )>,
    alias_target_transaction_depth: usize,
}

impl TypeContext {
    pub fn new() -> Self {
        let mut ctx = Self {
            types: Vec::new(),
            type_aliases: HashMap::new(),
            type_vars: Vec::new(),
            alias_instances: Vec::new(),
            alias_instance_ids: HashMap::new(),
            alias_target_undo: Vec::new(),
            alias_target_transaction_depth: 0,
        };
        // Pre-register primitive types at known positions
        ctx.intern(Ty::Never); // TypeId(0)
        ctx.intern(Ty::None); // TypeId(1)
        ctx.intern(Ty::Bool); // TypeId(2)
        ctx.intern(Ty::Int); // TypeId(3)
        ctx.intern(Ty::Float); // TypeId(4)
        ctx.intern(Ty::Str); // TypeId(5)
        ctx.intern(Ty::Error); // TypeId(6)
        ctx.intern(Ty::Any); // TypeId(7) — #240
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
        self.types
            .iter()
            .position(|t| t == ty)
            .map(|i| TypeId(i as u32))
    }

    // Well-known type IDs
    pub fn never(&self) -> TypeId {
        TypeId(0)
    }
    pub fn none(&self) -> TypeId {
        TypeId(1)
    }
    pub fn bool(&self) -> TypeId {
        TypeId(2)
    }
    pub fn int(&self) -> TypeId {
        TypeId(3)
    }
    pub fn float(&self) -> TypeId {
        TypeId(4)
    }
    pub fn str(&self) -> TypeId {
        TypeId(5)
    }
    pub fn error(&self) -> TypeId {
        TypeId(6)
    }
    pub fn any(&self) -> TypeId {
        TypeId(7)
    }

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

    // --- Recursive source and generated alias instances ---

    pub fn intern_alias_instance(
        &mut self,
        identity: AliasIdentity,
        name: String,
        args: Vec<TypeId>,
        display_arg_count: usize,
    ) -> (AliasInstanceId, TypeId) {
        debug_assert!(display_arg_count <= args.len());
        let key = (identity, args.clone());
        if let Some(id) = self.alias_instance_ids.get(&key).copied() {
            debug_assert_eq!(
                self.alias_instances[id.0 as usize].display_arg_count,
                display_arg_count
            );
            return (id, self.alias_instances[id.0 as usize].ty);
        }

        let id = AliasInstanceId(self.alias_instances.len() as u32);
        let ty = self.intern(Ty::AliasRef(id));
        self.alias_instances.push(AliasInstance {
            identity,
            name,
            args,
            display_arg_count,
            target: None,
            deferred_target: None,
            rejected: false,
            resolving: false,
            ty,
        });
        self.alias_instance_ids.insert(key, id);
        (id, ty)
    }

    pub fn alias_instance(&self, id: AliasInstanceId) -> &AliasInstance {
        &self.alias_instances[id.0 as usize]
    }

    pub fn alias_target(&self, id: AliasInstanceId) -> Option<TypeId> {
        self.alias_instance(id).target
    }

    pub fn alias_target_is_resolving(&self, id: AliasInstanceId) -> bool {
        self.alias_instance(id).resolving
    }

    pub fn alias_target_is_rejected(&self, id: AliasInstanceId) -> bool {
        self.alias_instance(id).rejected
    }

    pub fn alias_ref_is_rejected(&self, ty: TypeId) -> bool {
        matches!(self.get(ty), Ty::AliasRef(id) if self.alias_target_is_rejected(*id))
    }

    pub fn deferred_alias_target(&self, id: AliasInstanceId) -> Option<&DeferredAliasTarget> {
        self.alias_instance(id).deferred_target.as_ref()
    }

    pub fn begin_alias_target_transaction(&mut self) -> usize {
        self.alias_target_transaction_depth += 1;
        self.alias_target_undo.len()
    }

    pub fn finish_alias_target_transaction(&mut self, checkpoint: usize, commit: bool) {
        debug_assert!(self.alias_target_transaction_depth > 0);
        debug_assert!(checkpoint <= self.alias_target_undo.len());
        if commit {
            if self.alias_target_transaction_depth == 1 {
                self.alias_target_undo.truncate(checkpoint);
            }
        } else {
            let changes: Vec<_> = self.alias_target_undo.drain(checkpoint..).collect();
            for (id, target, deferred_target, rejected, resolving) in changes.into_iter().rev() {
                let instance = &mut self.alias_instances[id.0 as usize];
                instance.target = target;
                instance.deferred_target = deferred_target;
                instance.rejected = rejected;
                instance.resolving = resolving;
            }
        }
        self.alias_target_transaction_depth -= 1;
    }

    fn record_alias_target_change(&mut self, id: AliasInstanceId) -> bool {
        if self.alias_target_transaction_depth == 0 {
            return false;
        }
        let instance = &self.alias_instances[id.0 as usize];
        self.alias_target_undo
            .push((
                id,
                instance.target,
                instance.deferred_target.clone(),
                instance.rejected,
                instance.resolving,
            ));
        true
    }

    pub fn begin_alias_target(&mut self, id: AliasInstanceId) -> bool {
        let recorded = self.record_alias_target_change(id);
        let instance = &mut self.alias_instances[id.0 as usize];
        if instance.target.is_some() || instance.resolving {
            if recorded {
                self.alias_target_undo.pop();
            }
            return false;
        }
        instance.resolving = true;
        true
    }

    /// Record how to finish a transformed specialization once its declaration
    /// instance has completed resolving. This is deliberately state in the
    /// TypeContext so generated-alias materialization can be transactional.
    pub fn defer_alias_target(
        &mut self,
        id: AliasInstanceId,
        template: AliasInstanceId,
        substitutions: Vec<(TypeVarId, TypeId)>,
        param_packs: Vec<(TypeVarId, ParamPack)>,
    ) {
        let _ = self.record_alias_target_change(id);
        let instance = &mut self.alias_instances[id.0 as usize];
        debug_assert!(instance.target.is_none());
        debug_assert!(!instance.resolving);
        instance.rejected = false;
        let deferred = DeferredAliasTarget {
            template,
            substitutions,
            param_packs,
        };
        if let Some(existing) = &instance.deferred_target {
            debug_assert_eq!(existing, &deferred);
        } else {
            instance.deferred_target = Some(deferred);
        }
    }

    pub fn set_alias_target(&mut self, id: AliasInstanceId, target: TypeId) {
        let _ = self.record_alias_target_change(id);
        let instance = &mut self.alias_instances[id.0 as usize];
        if let Some(existing) = instance.target {
            debug_assert_eq!(
                existing, target,
                "alias target changed after materialization"
            );
            instance.resolving = false;
            return;
        }
        instance.target = Some(target);
        instance.deferred_target = None;
        instance.rejected = false;
        instance.resolving = false;
    }

    /// Discard an alias expansion that cannot be assigned a productive target.
    pub fn abandon_alias_target(&mut self, id: AliasInstanceId) {
        let _ = self.record_alias_target_change(id);
        let instance = &mut self.alias_instances[id.0 as usize];
        instance.target = None;
        instance.deferred_target = None;
        instance.rejected = false;
        instance.resolving = false;
    }

    /// Permanently close an alias expansion that cannot produce a sound
    /// target. `Never` is the fail-closed semantic target: unlike `Error`, it
    /// cannot make an arbitrary value satisfy the alias contract.
    pub fn reject_alias_target(&mut self, id: AliasInstanceId) {
        let never = self.never();
        let _ = self.record_alias_target_change(id);
        let instance = &mut self.alias_instances[id.0 as usize];
        instance.target = Some(never);
        instance.deferred_target = None;
        instance.rejected = true;
        instance.resolving = false;
    }

    /// Resolve only top-level alias indirections, preserving recursive edges
    /// nested under a productive constructor such as list or tuple.
    pub fn semantic_head_id(&self, ty: TypeId) -> Result<TypeId, AliasHeadError> {
        let mut current = ty;
        let mut seen = HashSet::new();
        loop {
            let Ty::AliasRef(id) = self.get(current) else {
                return Ok(current);
            };
            if self.alias_target_is_rejected(*id) {
                return Err(AliasHeadError::Rejected(*id));
            }
            if !seen.insert(*id) {
                return Err(AliasHeadError::Cycle(*id));
            }
            current = self
                .alias_target(*id)
                .ok_or(AliasHeadError::Unresolved(*id))?;
        }
    }

    pub fn semantic_ty_or_error(&self, ty: TypeId) -> &Ty {
        let head = match self.semantic_head_id(ty) {
            Ok(head) => head,
            Err(AliasHeadError::Rejected(_)) => self.never(),
            Err(AliasHeadError::Unresolved(_) | AliasHeadError::Cycle(_)) => self.error(),
        };
        self.get(head)
    }

    /// Whether `target` reaches the same alias without crossing a structural
    /// constructor. Alias forwarding and unions are transparent here: neither
    /// consumes a runtime layer, so `A = int | A` is not contractive.
    pub fn alias_has_unguarded_cycle(&self, origin: AliasInstanceId, target: TypeId) -> bool {
        fn visit(
            tcx: &TypeContext,
            origin_identity: AliasIdentity,
            current: TypeId,
            seen: &mut HashSet<AliasInstanceId>,
        ) -> bool {
            match tcx.get(current) {
                Ty::AliasRef(id) => {
                    if tcx.alias_instance(*id).identity == origin_identity {
                        return true;
                    }
                    if !seen.insert(*id) {
                        return false;
                    }
                    let reaches = tcx
                        .alias_target(*id)
                        .is_some_and(|target| visit(tcx, origin_identity, target, seen));
                    seen.remove(id);
                    reaches
                }
                Ty::Union(items) => items
                    .iter()
                    .any(|item| visit(tcx, origin_identity, *item, seen)),
                _ => false,
            }
        }

        let origin_identity = self.alias_instance(origin).identity;
        let mut seen = HashSet::new();
        visit(self, origin_identity, target, &mut seen)
    }

    /// Whether a candidate target reaches a rejected alias or a generated
    /// alias that still has no semantic target. The origin itself is the one
    /// valid exception: once installed, that edge is a finite back-edge.
    pub fn alias_target_has_invalid_generated_edge(
        &self,
        origin: AliasInstanceId,
        target: TypeId,
    ) -> bool {
        fn visit(
            tcx: &TypeContext,
            origin: AliasInstanceId,
            current: TypeId,
            seen: &mut HashSet<AliasInstanceId>,
        ) -> bool {
            match tcx.get(current) {
                Ty::AliasRef(id) => {
                    if *id == origin {
                        return false;
                    }
                    if tcx.alias_target_is_rejected(*id) {
                        return true;
                    }
                    if !seen.insert(*id) {
                        return false;
                    }
                    let unresolved = match tcx.alias_target(*id) {
                        Some(target) => visit(tcx, origin, target, seen),
                        None => matches!(
                            tcx.alias_instance(*id).identity,
                            AliasIdentity::Generated(_, _)
                        ),
                    };
                    seen.remove(id);
                    unresolved
                }
                Ty::List(item) | Ty::Set(item) | Ty::TypeObject(item) => {
                    visit(tcx, origin, *item, seen)
                }
                Ty::Dict(key, value) => {
                    visit(tcx, origin, *key, seen) || visit(tcx, origin, *value, seen)
                }
                Ty::Tuple(items) | Ty::Union(items) => {
                    items.iter().any(|item| visit(tcx, origin, *item, seen))
                }
                Ty::Fn {
                    params,
                    ret,
                    signature,
                    ..
                } => {
                    params.iter().any(|param| visit(tcx, origin, *param, seen))
                        || signature.as_ref().is_some_and(|params| {
                            params
                                .iter()
                                .any(|param| visit(tcx, origin, param.ty, seen))
                        })
                        || visit(tcx, origin, *ret, seen)
                }
                Ty::External(ExternalValue::Callable(callable)) => callable
                    .receiver
                    .as_ref()
                    .is_some_and(|receiver| {
                        receiver
                            .args
                            .iter()
                            .any(|arg| visit(tcx, origin, *arg, seen))
                    }),
                Ty::Class {
                    user,
                    external,
                    fields,
                    ..
                } => {
                    user.as_ref().is_some_and(|user| {
                        user.args
                            .iter()
                            .any(|arg| visit(tcx, origin, *arg, seen))
                    }) || external.as_ref().is_some_and(|external| {
                        external
                            .args
                            .iter()
                            .any(|arg| visit(tcx, origin, *arg, seen))
                    }) || fields
                        .iter()
                        .any(|(_, field)| visit(tcx, origin, *field, seen))
                }
                Ty::Enum { variants, .. } => variants.iter().any(|(_, fields)| {
                    fields
                        .iter()
                        .any(|field| visit(tcx, origin, *field, seen))
                }),
                Ty::Never
                | Ty::None
                | Ty::Bool
                | Ty::Int
                | Ty::Float
                | Ty::Str
                | Ty::Any
                | Ty::TypeVar(_)
                | Ty::External(ExternalValue::Module { .. })
                | Ty::Literal(_)
                | Ty::SelfType
                | Ty::Infer(_)
                | Ty::Error => false,
            }
        }

        visit(self, origin, target, &mut HashSet::new())
    }

    // --- Type variables (#242) ---

    pub fn new_type_var(
        &mut self,
        name: String,
        bound: Option<TypeId>,
        constraints: Vec<TypeId>,
    ) -> TypeVarId {
        self.new_type_param(
            name,
            TypeVarKind::TypeVar,
            bound,
            constraints,
            TypeParamDefault::None,
        )
    }

    pub fn new_type_param(
        &mut self,
        name: String,
        kind: TypeVarKind,
        bound: Option<TypeId>,
        constraints: Vec<TypeId>,
        default: TypeParamDefault,
    ) -> TypeVarId {
        let id = TypeVarId(self.type_vars.len() as u32);
        self.type_vars.push(TypeVarInfo {
            name,
            kind,
            bound,
            constraints,
            default,
        });
        id
    }

    pub fn get_type_var(&self, id: TypeVarId) -> &TypeVarInfo {
        &self.type_vars[id.0 as usize]
    }

    /// Fill metadata after a declaration's TypeVars have all been allocated.
    /// Allocation and resolution are separate so lazy forward metadata keeps
    /// the stable TypeVar identity embedded in previously interned types.
    pub fn set_type_var_metadata(
        &mut self,
        id: TypeVarId,
        bound: Option<TypeId>,
        constraints: Vec<TypeId>,
        default: TypeParamDefault,
    ) {
        let info = &mut self.type_vars[id.0 as usize];
        info.bound = bound;
        info.constraints = constraints;
        info.default = default;
    }

    /// Whether a type is still parameterized by a TypeVar-like placeholder.
    /// PEP 695 bounds and constraints must be concrete after name resolution.
    pub fn contains_type_var(&self, id: TypeId) -> bool {
        match self.get(id) {
            Ty::TypeVar(_) | Ty::SelfType | Ty::Infer(_) => true,
            Ty::AliasRef(id) => self
                .alias_instance(*id)
                .args
                .iter()
                .any(|arg| self.contains_type_var(*arg)),
            Ty::List(item) | Ty::Set(item) | Ty::TypeObject(item) => {
                self.contains_type_var(*item)
            }
            Ty::Dict(key, value) => self.contains_type_var(*key) || self.contains_type_var(*value),
            Ty::Tuple(items) | Ty::Union(items) => {
                items.iter().any(|item| self.contains_type_var(*item))
            }
            Ty::Fn {
                params,
                ret,
                signature,
                param_spec,
                ..
            } => {
                params.iter().any(|param| self.contains_type_var(*param))
                    || signature.as_ref().is_some_and(|params| {
                        params
                            .iter()
                            .any(|param| self.contains_type_var(param.ty))
                    })
                    || self.contains_type_var(*ret)
                    || param_spec.is_some()
            }
            Ty::External(ExternalValue::Callable(callable)) => callable
                .receiver
                .as_ref()
                .is_some_and(|receiver| {
                    receiver
                        .args
                        .iter()
                        .any(|arg| self.contains_type_var(*arg))
                }),
            Ty::Class {
                user,
                external,
                fields,
                ..
            } => {
                user.as_ref()
                    .is_some_and(|user| user.args.iter().any(|arg| self.contains_type_var(*arg)))
                    || external.as_ref().is_some_and(|external| {
                        external
                            .args
                            .iter()
                            .any(|arg| self.contains_type_var(*arg))
                    })
                    || fields
                        .iter()
                        .any(|(_, field)| self.contains_type_var(*field))
            }
            Ty::Enum { variants, .. } => variants
                .iter()
                .any(|(_, fields)| fields.iter().any(|field| self.contains_type_var(*field))),
            Ty::Never
            | Ty::None
            | Ty::Bool
            | Ty::Int
            | Ty::Float
            | Ty::Str
            | Ty::Any
            | Ty::External(ExternalValue::Module { .. })
            | Ty::Literal(_)
            | Ty::Error => false,
        }
    }

    /// Collect the TypeVar identities referenced anywhere inside a type.
    pub fn type_vars_in(&self, id: TypeId) -> Vec<TypeVarId> {
        let mut vars = Vec::new();
        self.collect_type_vars(id, &mut vars);
        vars.sort_unstable_by_key(|var| var.0);
        vars.dedup();
        vars
    }

    fn collect_type_vars(&self, id: TypeId, vars: &mut Vec<TypeVarId>) {
        match self.get(id) {
            Ty::TypeVar(var) => vars.push(*var),
            Ty::AliasRef(id) => {
                for arg in &self.alias_instance(*id).args {
                    self.collect_type_vars(*arg, vars);
                }
            }
            Ty::List(item) | Ty::Set(item) | Ty::TypeObject(item) => {
                self.collect_type_vars(*item, vars)
            }
            Ty::Dict(key, value) => {
                self.collect_type_vars(*key, vars);
                self.collect_type_vars(*value, vars);
            }
            Ty::Tuple(items) | Ty::Union(items) => {
                for item in items {
                    self.collect_type_vars(*item, vars);
                }
            }
            Ty::Fn {
                params,
                ret,
                signature,
                param_spec,
                ..
            } => {
                for param in params {
                    self.collect_type_vars(*param, vars);
                }
                if let Some(params) = signature {
                    for param in params {
                        self.collect_type_vars(param.ty, vars);
                    }
                }
                if let Some(param_spec) = param_spec {
                    vars.push(*param_spec);
                }
                self.collect_type_vars(*ret, vars);
            }
            Ty::External(ExternalValue::Callable(callable)) => {
                if let Some(receiver) = &callable.receiver {
                    for arg in &receiver.args {
                        self.collect_type_vars(*arg, vars);
                    }
                }
            }
            Ty::Class {
                user,
                external,
                fields,
                ..
            } => {
                if let Some(user) = user {
                    for arg in &user.args {
                        self.collect_type_vars(*arg, vars);
                    }
                }
                if let Some(external) = external {
                    for arg in &external.args {
                        self.collect_type_vars(*arg, vars);
                    }
                }
                for (_, field) in fields {
                    self.collect_type_vars(*field, vars);
                }
            }
            Ty::Enum { variants, .. } => {
                for (_, fields) in variants {
                    for field in fields {
                        self.collect_type_vars(*field, vars);
                    }
                }
            }
            Ty::Never
            | Ty::None
            | Ty::Bool
            | Ty::Int
            | Ty::Float
            | Ty::Str
            | Ty::Any
            | Ty::External(ExternalValue::Module { .. })
            | Ty::Literal(_)
            | Ty::SelfType
            | Ty::Infer(_)
            | Ty::Error => {}
        }
    }

    // --- Subtype checking ---

    /// Check if `sub` is a subtype of `sup` (simplified).
    pub fn is_subtype(&self, sub: TypeId, sup: TypeId) -> bool {
        let mut visiting = HashSet::new();
        self.is_subtype_inner(sub, sup, &mut visiting)
    }

    fn is_subtype_inner(
        &self,
        sub: TypeId,
        sup: TypeId,
        visiting: &mut HashSet<(TypeId, TypeId)>,
    ) -> bool {
        if self.alias_ref_is_rejected(sub) || self.alias_ref_is_rejected(sup) {
            return false;
        }
        if sub == sup {
            return true;
        }

        if !visiting.insert((sub, sup)) {
            return true;
        }
        let result = self.is_subtype_step(sub, sup, visiting);
        visiting.remove(&(sub, sup));
        result
    }

    fn is_subtype_step(
        &self,
        sub: TypeId,
        sup: TypeId,
        visiting: &mut HashSet<(TypeId, TypeId)>,
    ) -> bool {
        let sub_ty = self.get(sub);
        let sup_ty = self.get(sup);

        if let Ty::AliasRef(id) = sub_ty {
            return self
                .alias_target(*id)
                .is_some_and(|target| self.is_subtype_inner(target, sup, visiting));
        }
        if let Ty::AliasRef(id) = sup_ty {
            return self
                .alias_target(*id)
                .is_some_and(|target| self.is_subtype_inner(sub, target, visiting));
        }

        // Any is compatible with everything
        if matches!(sup_ty, Ty::Any) || matches!(sub_ty, Ty::Any) {
            return true;
        }

        // Never is subtype of everything
        if matches!(sub_ty, Ty::Never) {
            return true;
        }

        if let (Ty::TypeObject(sub_instance), Ty::TypeObject(sup_instance)) = (sub_ty, sup_ty) {
            return self.is_subtype_inner(*sub_instance, *sup_instance, visiting);
        }

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
            return variants
                .iter()
                .any(|v| self.is_subtype_inner(sub, *v, visiting));
        }

        // Sub union: all variants must be subtypes of sup
        if let Ty::Union(variants) = sub_ty {
            return variants
                .iter()
                .all(|v| self.is_subtype_inner(*v, sup, visiting));
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
    fn test_user_class_symbol_is_part_of_nominal_identity() {
        let mut tcx = TypeContext::new();
        let first = tcx.intern(Ty::Class {
            name: "Nested".to_string(),
            role: crate::types::ty::ClassRole::Instance,
            user: Some(crate::types::ty::UserClass {
                symbol: crate::resolve::SymbolId(10),
                args: Vec::new(),
            }),
            external: None,
            fields: Vec::new(),
            match_args: None,
        });
        let second = tcx.intern(Ty::Class {
            name: "Nested".to_string(),
            role: crate::types::ty::ClassRole::Instance,
            user: Some(crate::types::ty::UserClass {
                symbol: crate::resolve::SymbolId(11),
                args: Vec::new(),
            }),
            external: None,
            fields: Vec::new(),
            match_args: None,
        });

        assert_ne!(first, second);
    }

    #[test]
    fn test_class_role_is_part_of_identity() {
        let mut tcx = TypeContext::new();
        let object = tcx.intern(Ty::Class {
            name: "Box".to_string(),
            role: crate::types::ty::ClassRole::Object,
            user: Some(crate::types::ty::UserClass {
                symbol: crate::resolve::SymbolId(10),
                args: vec![tcx.int()],
            }),
            external: None,
            fields: Vec::new(),
            match_args: None,
        });
        let instance = tcx.intern(Ty::Class {
            name: "Box".to_string(),
            role: crate::types::ty::ClassRole::Instance,
            user: Some(crate::types::ty::UserClass {
                symbol: crate::resolve::SymbolId(10),
                args: vec![tcx.int()],
            }),
            external: None,
            fields: Vec::new(),
            match_args: None,
        });

        assert_ne!(object, instance);

        let native_object = tcx.intern(Ty::Class {
            name: "ValueError".to_string(),
            role: crate::types::ty::ClassRole::Object,
            user: None,
            external: None,
            fields: Vec::new(),
            match_args: None,
        });
        let native_instance = tcx.intern(Ty::Class {
            name: "ValueError".to_string(),
            role: crate::types::ty::ClassRole::Instance,
            user: None,
            external: None,
            fields: Vec::new(),
            match_args: None,
        });
        assert_ne!(native_object, native_instance);
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
        assert_eq!(info.kind, TypeVarKind::TypeVar);
        assert!(info.bound.is_none());
        assert!(info.constraints.is_empty());
        assert_eq!(info.default, TypeParamDefault::None);
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
        let id = tcx.new_type_var("T".to_string(), None, vec![int_ty, str_ty]);
        let info = tcx.get_type_var(id);
        assert_eq!(info.constraints, vec![int_ty, str_ty]);
    }

    #[test]
    fn test_set_type_var_metadata_after_allocation() {
        let mut tcx = TypeContext::new();
        let int_ty = tcx.int();
        let str_ty = tcx.str();
        let id = tcx.new_type_var("T".to_string(), None, Vec::new());

        tcx.set_type_var_metadata(
            id,
            Some(int_ty),
            vec![int_ty, str_ty],
            TypeParamDefault::None,
        );

        let info = tcx.get_type_var(id);
        assert_eq!(info.bound, Some(int_ty));
        assert_eq!(info.constraints, vec![int_ty, str_ty]);
    }

    #[test]
    fn test_type_parameter_kind_and_unresolved_default_are_preserved() {
        let mut tcx = TypeContext::new();
        let id = tcx.new_type_param(
            "Ts".to_string(),
            TypeVarKind::TypeVarTuple,
            None,
            Vec::new(),
            TypeParamDefault::Unresolved,
        );

        let info = tcx.get_type_var(id);
        assert_eq!(info.kind, TypeVarKind::TypeVarTuple);
        assert_eq!(info.default, TypeParamDefault::Unresolved);
    }

    #[test]
    fn test_contains_type_var_recurses_through_containers() {
        let mut tcx = TypeContext::new();
        let id = tcx.new_type_var("T".to_string(), None, Vec::new());
        let type_var = tcx.intern(Ty::TypeVar(id));
        let list_type_var = tcx.intern(Ty::List(type_var));
        let int_ty = tcx.int();
        let list_int = tcx.intern(Ty::List(int_ty));

        assert!(tcx.contains_type_var(type_var));
        assert!(tcx.contains_type_var(list_type_var));
        assert!(!tcx.contains_type_var(list_int));
        assert_eq!(tcx.type_vars_in(list_type_var), vec![id]);
        assert!(tcx.type_vars_in(list_int).is_empty());
    }

    #[test]
    fn recursive_alias_instances_are_stable_and_backfilled() {
        let mut tcx = TypeContext::new();
        let symbol = SymbolId(42);
        let (instance, alias_ref) = tcx.intern_alias_instance(
            AliasIdentity::Source(symbol),
            "Node".to_string(),
            Vec::new(),
            0,
        );
        let (same_instance, same_ref) = tcx.intern_alias_instance(
            AliasIdentity::Source(symbol),
            "Node".to_string(),
            Vec::new(),
            0,
        );
        assert_eq!(instance, same_instance);
        assert_eq!(alias_ref, same_ref);

        let target = tcx.intern(Ty::List(alias_ref));
        assert!(!tcx.alias_has_unguarded_cycle(instance, target));
        tcx.set_alias_target(instance, target);
        assert_eq!(tcx.semantic_head_id(alias_ref), Ok(target));
    }

    #[test]
    fn recursive_alias_head_cycles_are_reported_without_unfolding() {
        let mut tcx = TypeContext::new();
        let (instance, alias_ref) = tcx.intern_alias_instance(
            AliasIdentity::Source(SymbolId(7)),
            "Loop".to_string(),
            Vec::new(),
            0,
        );
        assert!(tcx.alias_has_unguarded_cycle(instance, alias_ref));
        tcx.set_alias_target(instance, alias_ref);
        assert_eq!(
            tcx.semantic_head_id(alias_ref),
            Err(AliasHeadError::Cycle(instance))
        );
    }

    #[test]
    fn recursive_alias_arguments_participate_in_type_var_queries() {
        let mut tcx = TypeContext::new();
        let var = tcx.new_type_var("T".to_string(), None, Vec::new());
        let var_ty = tcx.intern(Ty::TypeVar(var));
        let (_, alias_ref) = tcx.intern_alias_instance(
            AliasIdentity::Source(SymbolId(9)),
            "Chain".to_string(),
            vec![var_ty],
            1,
        );
        assert!(tcx.contains_type_var(alias_ref));
        assert_eq!(tcx.type_vars_in(alias_ref), vec![var]);
    }

    #[test]
    fn subtype_checks_unfold_alias_heads_cycle_safely() {
        let mut tcx = TypeContext::new();
        let (instance, alias_ref) = tcx.intern_alias_instance(
            AliasIdentity::Source(SymbolId(11)),
            "Number".to_string(),
            Vec::new(),
            0,
        );
        tcx.set_alias_target(instance, tcx.int());
        assert!(tcx.is_subtype(alias_ref, tcx.float()));
        assert!(!tcx.is_subtype(alias_ref, tcx.str()));
    }

    #[test]
    fn rejected_alias_heads_preserve_identity_and_fail_compatibility() {
        let mut tcx = TypeContext::new();
        let (instance, alias_ref) = tcx.intern_alias_instance(
            AliasIdentity::Generated(StrSpecId(4), StrSpecId(5)),
            "example.Rejected".to_string(),
            vec![tcx.int()],
            1,
        );
        tcx.reject_alias_target(instance);

        assert_eq!(
            tcx.semantic_head_id(alias_ref),
            Err(AliasHeadError::Rejected(instance))
        );
        assert!(matches!(tcx.semantic_ty_or_error(alias_ref), Ty::Never));
        assert!(!tcx.is_subtype(alias_ref, alias_ref));
        assert!(!tcx.is_subtype(alias_ref, tcx.int()));
        assert!(!tcx.is_subtype(tcx.int(), alias_ref));
    }

    #[test]
    fn generated_alias_cycles_can_be_abandoned_and_retried() {
        let mut tcx = TypeContext::new();
        let (left, left_ref) = tcx.intern_alias_instance(
            AliasIdentity::Generated(StrSpecId(1), StrSpecId(2)),
            "example.Left".to_string(),
            Vec::new(),
            0,
        );
        let (right, right_ref) = tcx.intern_alias_instance(
            AliasIdentity::Generated(StrSpecId(1), StrSpecId(3)),
            "example.Right".to_string(),
            Vec::new(),
            0,
        );

        let checkpoint = tcx.begin_alias_target_transaction();
        assert!(tcx.begin_alias_target(left));
        assert!(tcx.alias_has_unguarded_cycle(left, left_ref));
        assert!(tcx.begin_alias_target(right));
        tcx.set_alias_target(right, left_ref);
        assert!(tcx.alias_has_unguarded_cycle(left, right_ref));

        tcx.finish_alias_target_transaction(checkpoint, false);
        assert_eq!(tcx.alias_target(left), None);
        assert_eq!(tcx.alias_target(right), None);
        assert!(tcx.deferred_alias_target(left).is_none());
        assert!(tcx.begin_alias_target(left));
        assert!(tcx.begin_alias_target(right));
    }

    #[test]
    fn deferred_alias_targets_roll_back_with_their_transaction() {
        let mut tcx = TypeContext::new();
        let (template, _) = tcx.intern_alias_instance(
            AliasIdentity::Generated(StrSpecId(1), StrSpecId(2)),
            "example.Template".to_string(),
            Vec::new(),
            0,
        );
        let (specialized, _) = tcx.intern_alias_instance(
            AliasIdentity::Generated(StrSpecId(1), StrSpecId(2)),
            "example.Specialized".to_string(),
            vec![tcx.int()],
            1,
        );
        let outer = tcx.begin_alias_target_transaction();
        let inner = tcx.begin_alias_target_transaction();
        tcx.defer_alias_target(specialized, template, Vec::new(), Vec::new());
        assert!(tcx.deferred_alias_target(specialized).is_some());
        tcx.finish_alias_target_transaction(inner, true);
        assert!(tcx.deferred_alias_target(specialized).is_some());

        let deferred_before_rejection = tcx.deferred_alias_target(specialized).cloned();
        let rejection = tcx.begin_alias_target_transaction();
        tcx.reject_alias_target(specialized);
        assert!(tcx.alias_target_is_rejected(specialized));
        tcx.finish_alias_target_transaction(rejection, false);
        assert_eq!(tcx.alias_target(specialized), None);
        assert_eq!(
            tcx.deferred_alias_target(specialized),
            deferred_before_rejection.as_ref()
        );
        assert!(!tcx.alias_target_is_rejected(specialized));

        let rejection = tcx.begin_alias_target_transaction();
        tcx.reject_alias_target(specialized);
        assert!(tcx.alias_target_is_rejected(specialized));
        tcx.finish_alias_target_transaction(rejection, true);
        assert!(tcx.alias_target_is_rejected(specialized));

        tcx.finish_alias_target_transaction(outer, false);
        assert_eq!(tcx.alias_target(specialized), None);
        assert!(tcx.deferred_alias_target(specialized).is_none());
        assert!(!tcx.alias_target_is_rejected(specialized));
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

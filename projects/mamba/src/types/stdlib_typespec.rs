//! Lossless, generated typeshed callable contracts.
//!
//! The generated artifact is compact structured data rather than Rust source:
//! parsing hundreds of thousands of generated constant lines made every mamba
//! rebuild pay for typeshed. Strings and source locations are interned, while
//! unsupported annotation nodes remain explicit and distinct from `Any`.

use serde::Deserialize;
use std::collections::HashSet;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct StrSpecId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct TypeSpecId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct TypeParamSpecId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct SourceSpanSpecId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct TypeUseSpecId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct ClassSpecId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct TableRange(pub u32, pub u32);

impl TableRange {
    pub const EMPTY: Self = Self(0, 0);

    pub const fn new(start: u32, len: u32) -> Self {
        Self(start, len)
    }

    pub fn bounds(self) -> std::ops::Range<usize> {
        self.0 as usize..(self.0 + self.1) as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum TypeNameKind {
    #[serde(rename = "b")]
    Builtin,
    #[serde(rename = "s")]
    Special,
    #[serde(rename = "n")]
    Nominal,
    #[serde(rename = "p")]
    Protocol,
    #[serde(rename = "a")]
    Alias,
    #[serde(rename = "i")]
    Imported,
    #[serde(rename = "u")]
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum TypeSpecNode {
    Missing,
    Unsupported(StrSpecId),
    Any,
    Never,
    None,
    SelfType,
    Ellipsis,
    Name {
        module: StrSpecId,
        name: StrSpecId,
        kind: TypeNameKind,
    },
    TypeParam(TypeParamSpecId),
    Union(TableRange),
    Apply {
        base: TypeSpecId,
        args: TableRange,
    },
    Tuple(TableRange),
    ParamList(TableRange),
    Unpack(TypeSpecId),
    LiteralInt(i64),
    LiteralStr(StrSpecId),
    LiteralBool(bool),
    LiteralBytes(StrSpecId),
    LiteralNone,
    ForwardRef {
        expression: StrSpecId,
        target: TypeSpecId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum TypeParamSpecKind {
    #[serde(rename = "t")]
    TypeVar,
    #[serde(rename = "v")]
    TypeVarTuple,
    #[serde(rename = "p")]
    ParamSpec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum VarianceSpec {
    #[serde(rename = "i")]
    Invariant,
    #[serde(rename = "c")]
    Covariant,
    #[serde(rename = "d")]
    Contravariant,
    #[serde(rename = "f")]
    Infer,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TypeParamSpec {
    pub key: StrSpecId,
    pub name: StrSpecId,
    pub kind: TypeParamSpecKind,
    pub variance: VarianceSpec,
    pub bound: Option<TypeSpecId>,
    pub constraints: TableRange,
    pub default: Option<TypeSpecId>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AliasSpec {
    pub module: StrSpecId,
    pub qualifier: StrSpecId,
    pub name: StrSpecId,
    pub target: TypeSpecId,
    pub type_params: TableRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ClassSpecKind {
    #[serde(rename = "n")]
    Nominal,
    #[serde(rename = "p")]
    Protocol,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "ClassSpecWire")]
pub struct ClassSpec {
    pub module: StrSpecId,
    pub qualifier: StrSpecId,
    pub name: StrSpecId,
    pub kind: ClassSpecKind,
    pub type_params: TableRange,
    pub bases: TableRange,
    pub methods: TableRange,
    pub source: SourceSpanSpecId,
    pub method_only_complete: bool,
}

#[derive(Deserialize)]
struct ClassSpecWire(
    StrSpecId,
    StrSpecId,
    StrSpecId,
    ClassSpecKind,
    TableRange,
    TableRange,
    TableRange,
    SourceSpanSpecId,
    bool,
);

impl From<ClassSpecWire> for ClassSpec {
    fn from(value: ClassSpecWire) -> Self {
        Self {
            module: value.0,
            qualifier: value.1,
            name: value.2,
            kind: value.3,
            type_params: value.4,
            bases: value.5,
            methods: value.6,
            source: value.7,
            method_only_complete: value.8,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct ClassExportSpec(StrSpecId, StrSpecId, ClassSpecId);

#[derive(Debug, Clone, Copy, Deserialize)]
struct CallableExportSpec(StrSpecId, StrSpecId, StrSpecId, StrSpecId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct SourceSpanSpec(
    pub StrSpecId,
    pub u32,
    pub u32,
    pub u32,
    pub u32,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct TypeUseSpec(pub TypeSpecId, pub SourceSpanSpecId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ParamSpecKind {
    #[serde(rename = "p")]
    PosOnly,
    #[serde(rename = "r")]
    PosOrKw,
    #[serde(rename = "v")]
    VarPos,
    #[serde(rename = "k")]
    KwOnly,
    #[serde(rename = "w")]
    VarKw,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "ParamSpecWire")]
pub struct ParamSpec {
    pub name: StrSpecId,
    pub kind: ParamSpecKind,
    pub ty: TypeUseSpecId,
    pub has_default: bool,
    pub implicit_receiver: bool,
}

#[derive(Deserialize)]
struct ParamSpecWire(StrSpecId, ParamSpecKind, TypeUseSpecId, bool, bool);

impl From<ParamSpecWire> for ParamSpec {
    fn from(value: ParamSpecWire) -> Self {
        Self {
            name: value.0,
            kind: value.1,
            ty: value.2,
            has_default: value.3,
            implicit_receiver: value.4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum CallableSpecKind {
    #[serde(rename = "m")]
    ModuleFn,
    #[serde(rename = "i")]
    InstanceMethod,
    #[serde(rename = "c")]
    ClassMethod,
    #[serde(rename = "s")]
    StaticMethod,
    #[serde(rename = "g")]
    PropertyGet,
    #[serde(rename = "t")]
    PropertySet,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "GuardSpecWire")]
pub struct GuardSpec {
    pub expression: StrSpecId,
    pub polarity: bool,
    pub py312: bool,
}

#[derive(Deserialize)]
struct GuardSpecWire(StrSpecId, bool, bool);

impl From<GuardSpecWire> for GuardSpec {
    fn from(value: GuardSpecWire) -> Self {
        Self {
            expression: value.0,
            polarity: value.1,
            py312: value.2,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "CallableSpecWire")]
pub struct CallableSpec {
    pub module: StrSpecId,
    pub qualifier: StrSpecId,
    pub name: StrSpecId,
    pub kind: CallableSpecKind,
    pub params: TableRange,
    pub type_params: TableRange,
    pub ret: TypeUseSpecId,
    pub decorators: TableRange,
    pub guards: TableRange,
    pub source: SourceSpanSpecId,
    pub is_async: bool,
    pub branch: u16,
    pub py312: bool,
}

#[derive(Deserialize)]
struct CallableSpecWire(
    StrSpecId,
    StrSpecId,
    StrSpecId,
    CallableSpecKind,
    TableRange,
    TableRange,
    TypeUseSpecId,
    TableRange,
    TableRange,
    SourceSpanSpecId,
    bool,
    u16,
    bool,
);

impl From<CallableSpecWire> for CallableSpec {
    fn from(value: CallableSpecWire) -> Self {
        Self {
            module: value.0,
            qualifier: value.1,
            name: value.2,
            kind: value.3,
            params: value.4,
            type_params: value.5,
            ret: value.6,
            decorators: value.7,
            guards: value.8,
            source: value.9,
            is_async: value.10,
            branch: value.11,
            py312: value.12,
        }
    }
}

#[derive(Debug, Deserialize)]
struct StdlibSpecManifest {
    schema: u32,
    strings: Vec<String>,
    nodes: Vec<TypeSpecNode>,
    edges: Vec<TypeSpecId>,
    type_params: Vec<TypeParamSpec>,
    type_param_edges: Vec<TypeParamSpecId>,
    aliases: Vec<AliasSpec>,
    classes: Vec<ClassSpec>,
    class_method_edges: Vec<u32>,
    class_exports: Vec<ClassExportSpec>,
    callable_exports: Vec<CallableExportSpec>,
    class_callables: Vec<CallableSpec>,
    source_spans: Vec<SourceSpanSpec>,
    type_uses: Vec<TypeUseSpec>,
    params: Vec<ParamSpec>,
    decorators: Vec<StrSpecId>,
    guards: Vec<GuardSpec>,
    callables: Vec<CallableSpec>,
}

static MANIFEST: LazyLock<StdlibSpecManifest> = LazyLock::new(|| {
    let manifest: StdlibSpecManifest =
        serde_json::from_str(super::stdlib_specs_generated::MANIFEST_JSON)
            .expect("generated typeshed TypeSpec manifest must be valid");
    assert_eq!(manifest.schema, 2, "unsupported typeshed TypeSpec schema");
    manifest
});

pub fn string(id: StrSpecId) -> &'static str {
    &MANIFEST.strings[id.0 as usize]
}

pub fn node(id: TypeSpecId) -> &'static TypeSpecNode {
    &MANIFEST.nodes[id.0 as usize]
}

pub fn edges(range: TableRange) -> &'static [TypeSpecId] {
    &MANIFEST.edges[range.bounds()]
}

pub fn params(range: TableRange) -> &'static [ParamSpec] {
    &MANIFEST.params[range.bounds()]
}

pub fn type_param_edges(range: TableRange) -> &'static [TypeParamSpecId] {
    &MANIFEST.type_param_edges[range.bounds()]
}

pub fn type_param(id: TypeParamSpecId) -> &'static TypeParamSpec {
    &MANIFEST.type_params[id.0 as usize]
}

pub fn alias(module: &str, name: &str) -> Option<&'static AliasSpec> {
    MANIFEST.aliases.iter().find(|alias| {
        string(alias.module) == module
            && string(alias.qualifier).is_empty()
            && string(alias.name) == name
    })
}

pub fn class_by_id(id: ClassSpecId) -> &'static ClassSpec {
    &MANIFEST.classes[id.0 as usize]
}

pub fn class_spec(module: &str, name: &str) -> Option<(ClassSpecId, &'static ClassSpec)> {
    let export = MANIFEST.class_exports.iter().find(|export| {
        string(export.0) == module && string(export.1) == name
    })?;
    Some((export.2, class_by_id(export.2)))
}

fn class_spec_canonical(module: &str, qualifier: &str) -> Option<(ClassSpecId, &'static ClassSpec)> {
    MANIFEST
        .classes
        .iter()
        .enumerate()
        .find(|(_, class)| {
            string(class.module) == module && string(class.qualifier) == qualifier
        })
        .map(|(index, class)| (ClassSpecId(index as u32), class))
}

fn class_spec_any_name(module: &str, name: &str) -> Option<(ClassSpecId, &'static ClassSpec)> {
    class_spec(module, name).or_else(|| class_spec_canonical(module, name))
}

/// Whether the generated Python 3.12 corpus contains a usable namespace for
/// this module. Modules with only constants intentionally remain dynamic.
pub fn module_exists(module: &str) -> bool {
    MANIFEST
        .class_exports
        .iter()
        .any(|export| string(export.0) == module)
        || MANIFEST
            .callable_exports
            .iter()
            .any(|export| string(export.0) == module)
        || MANIFEST.aliases.iter().any(|alias| {
            string(alias.module) == module && string(alias.qualifier).is_empty()
        })
        || MANIFEST.classes.iter().any(|class| string(class.module) == module)
        || MANIFEST
            .callables
            .iter()
            .any(|callable| callable.py312 && string(callable.module) == module)
}

/// Resolve a public class export to its canonical generated identity.
pub fn exported_class(module: &str, name: &str) -> Option<(&'static str, &'static str)> {
    let (_, class) = class_spec(module, name)?;
    Some((string(class.module), string(class.qualifier)))
}

/// Whether a public module member has a generated module-function contract.
pub fn module_callable_exists(module: &str, name: &str) -> bool {
    overloads(module, "", name).any(|sig| sig.kind == CallableSpecKind::ModuleFn)
}

pub fn class_bases(class: &ClassSpec) -> &'static [TypeSpecId] {
    edges(class.bases)
}

pub fn class_type_params(class: &ClassSpec) -> &'static [TypeParamSpecId] {
    type_param_edges(class.type_params)
}

pub fn class_methods(
    class: &ClassSpec,
) -> impl Iterator<Item = &'static CallableSpec> + use<> {
    MANIFEST.class_method_edges[class.methods.bounds()]
        .iter()
        .map(|id| &MANIFEST.class_callables[*id as usize])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassBaseStep {
    pub child: ClassSpecId,
    pub base: ClassSpecId,
    pub args: TableRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassCallableResolution {
    pub owner: ClassSpecId,
    pub path: Vec<ClassBaseStep>,
}

fn class_base_step(child: ClassSpecId, spec_id: TypeSpecId) -> Option<ClassBaseStep> {
    let (module, name, args) = match node(spec_id) {
        TypeSpecNode::Name { module, name, .. } => (*module, *name, TableRange::EMPTY),
        TypeSpecNode::Apply { base, args } => match node(*base) {
            TypeSpecNode::Name { module, name, .. } => (*module, *name, *args),
            _ => return None,
        },
        _ => return None,
    };
    let (base, _) = class_spec_any_name(string(module), string(name))?;
    Some(ClassBaseStep { child, base, args })
}

/// Resolve a class callable and retain the generic inheritance path from the
/// requested class to its declaration owner. Distinct paths remain ambiguous
/// even when they reach the same owner because they may project type arguments
/// differently.
pub fn class_callable_resolution(
    module: &str,
    qualifier: &str,
    name: &str,
    kinds: &[CallableSpecKind],
) -> Option<ClassCallableResolution> {
    fn visit(
        class_id: ClassSpecId,
        name: &str,
        kinds: &[CallableSpecKind],
        visiting: &mut HashSet<ClassSpecId>,
    ) -> Result<Option<ClassCallableResolution>, ()> {
        if !visiting.insert(class_id) {
            return Err(());
        }
        let result = (|| {
            let class = class_by_id(class_id);
            if class_methods(class).any(|method| {
                method.py312 && string(method.name) == name && kinds.contains(&method.kind)
            }) {
                return Ok(Some(ClassCallableResolution {
                    owner: class_id,
                    path: Vec::new(),
                }));
            }

            let mut found: Option<ClassCallableResolution> = None;
            for base in class_bases(class) {
                let step = class_base_step(class_id, *base).ok_or(())?;
                if let Some(mut resolution) = visit(step.base, name, kinds, visiting)? {
                    resolution.path.insert(0, step);
                    if found
                        .as_ref()
                        .is_some_and(|existing| existing != &resolution)
                    {
                        return Err(());
                    }
                    found = Some(resolution);
                }
            }
            Ok(found)
        })();
        visiting.remove(&class_id);
        result
    }

    let (class_id, _) = class_spec_any_name(module, qualifier)?;
    visit(class_id, name, kinds, &mut HashSet::new())
        .ok()
        .flatten()
}

/// Resolve the declaration owner for a class callable, walking generated base
/// classes conservatively. A direct declaration wins; ambiguous or incomplete
/// multiple-inheritance paths return `None` instead of guessing an MRO.
pub fn class_callable_owner(
    module: &str,
    qualifier: &str,
    name: &str,
    kinds: &[CallableSpecKind],
) -> Option<(&'static str, &'static str)> {
    let resolution = class_callable_resolution(module, qualifier, name, kinds)?;
    let owner = class_by_id(resolution.owner);
    Some((string(owner.module), string(owner.qualifier)))
}

pub fn decorators(range: TableRange) -> impl Iterator<Item = &'static str> {
    MANIFEST.decorators[range.bounds()].iter().map(|id| string(*id))
}

pub fn guards(range: TableRange) -> &'static [GuardSpec] {
    &MANIFEST.guards[range.bounds()]
}

pub fn type_use(id: TypeUseSpecId) -> TypeUseSpec {
    MANIFEST.type_uses[id.0 as usize]
}

pub fn source_span(id: SourceSpanSpecId) -> SourceSpanSpec {
    MANIFEST.source_spans[id.0 as usize]
}

pub fn source_file(id: StrSpecId) -> &'static str {
    string(id)
}

pub fn overloads<'a>(
    module: &'a str,
    qualifier: &'a str,
    name: &'a str,
) -> impl Iterator<Item = &'static CallableSpec> + 'a {
    let (module, qualifier, name) = if qualifier.is_empty() {
        MANIFEST
            .callable_exports
            .iter()
            .find(|export| string(export.0) == module && string(export.1) == name)
            .map(|export| {
                (
                    string(export.2).to_string(),
                    String::new(),
                    string(export.3).to_string(),
                )
            })
            .unwrap_or_else(|| (module.to_string(), String::new(), name.to_string()))
    } else if let Some((_id, class)) = class_spec(module, qualifier) {
        (
            string(class.module).to_string(),
            string(class.qualifier).to_string(),
            name.to_string(),
        )
    } else {
        (module.to_string(), qualifier.to_string(), name.to_string())
    };
    MANIFEST.callables.iter().filter(move |sig| {
        sig.py312
            && string(sig.module) == module.as_str()
            && string(sig.qualifier) == qualifier.as_str()
            && string(sig.name) == name.as_str()
    })
}

pub fn class_has_callable(module: &str, qualifier: &str) -> bool {
    MANIFEST.callables.iter().any(|sig| {
        sig.py312
            && string(sig.module) == module
            && string(sig.qualifier) == qualifier
            && sig.kind != CallableSpecKind::ModuleFn
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_manifest_preserves_copy_typevar_identity() {
        let sig = overloads("copy", "", "copy").next().expect("copy.copy spec");
        let [param] = params(sig.params) else {
            panic!("copy.copy must have one parameter");
        };
        let param_ty = type_use(param.ty).0;
        let TypeSpecNode::TypeParam(param_id) = node(param_ty) else {
            panic!("copy.copy parameter must retain its TypeVar");
        };
        assert_eq!(string(type_param(*param_id).name), "_T");
        assert_eq!(node(type_use(sig.ret).0), node(param_ty));
    }

    #[test]
    fn generated_manifest_preserves_overload_branches_and_parameter_kinds() {
        let branches: Vec<_> = overloads("builtins", "bytes", "__getitem__").collect();
        assert_eq!(branches.len(), 2);
        for branch in branches {
            let visible: Vec<_> = params(branch.params)
                .iter()
                .filter(|param| !param.implicit_receiver)
                .collect();
            assert_eq!(visible.len(), 1);
            assert_eq!(visible[0].kind, ParamSpecKind::PosOnly);
        }
    }

    #[test]
    fn generated_manifest_owns_positional_only_builtin_contracts() {
        for name in [
            "chr",
            "ord",
            "getattr",
            "hasattr",
            "setattr",
            "format",
            "isinstance",
            "issubclass",
        ] {
            let branches: Vec<_> = overloads("builtins", "", name).collect();
            assert!(!branches.is_empty(), "missing builtins.{name} contract");
            for branch in branches {
                let regular: Vec<_> = params(branch.params)
                    .iter()
                    .filter(|param| {
                        !param.implicit_receiver
                            && !matches!(param.kind, ParamSpecKind::VarPos | ParamSpecKind::VarKw)
                    })
                    .collect();
                assert!(!regular.is_empty(), "builtins.{name} has no parameters");
                assert!(
                    regular
                        .iter()
                        .all(|param| param.kind == ParamSpecKind::PosOnly),
                    "builtins.{name} must remain positional-only"
                );
            }
        }
    }

    #[test]
    fn generated_manifest_canonicalizes_collections_abc_exports() {
        for name in ["Iterable", "Sequence", "Mapping"] {
            let (_, class) = class_spec("collections.abc", name)
                .unwrap_or_else(|| panic!("collections.abc.{name} class spec"));
            assert_eq!(string(class.module), "typing");
            assert_eq!(string(class.qualifier), name);
        }
    }

    #[test]
    fn generated_manifest_canonicalizes_plain_class_aliases() {
        assert_eq!(
            exported_class("types", "LambdaType"),
            Some(("types", "FunctionType")),
        );
        assert_eq!(
            exported_class("types", "BuiltinMethodType"),
            Some(("types", "BuiltinFunctionType")),
        );
    }

    #[test]
    fn generated_manifest_canonicalizes_public_callable_exports() {
        let sig = overloads("operator", "", "index")
            .next()
            .expect("operator.index canonical callable");
        assert_eq!(string(sig.module), "_operator");
        assert_eq!(string(sig.name), "index");
    }

    #[test]
    fn generated_manifest_preserves_complete_supports_index_protocol() {
        let (_, class) = class_spec("typing", "SupportsIndex").expect("SupportsIndex spec");
        assert_eq!(class.kind, ClassSpecKind::Protocol);
        assert!(class.method_only_complete);
        let methods: Vec<_> = class_methods(class)
            .filter(|method| method.py312)
            .collect();
        assert_eq!(methods.len(), 1);
        assert_eq!(string(methods[0].name), "__index__");
    }

    #[test]
    fn generated_manifest_resolves_direct_and_inherited_class_callable_owners() {
        let method_kinds = [
            CallableSpecKind::InstanceMethod,
            CallableSpecKind::ClassMethod,
            CallableSpecKind::StaticMethod,
        ];
        assert_eq!(
            class_callable_owner("builtins", "set", "__ior__", &method_kinds),
            Some(("builtins", "set")),
        );
        assert_eq!(
            class_callable_owner("pathlib", "Path", "as_posix", &method_kinds),
            Some(("pathlib", "PurePath")),
        );
    }

    #[test]
    fn generated_manifest_retains_inherited_generic_projection_path() {
        let method_kinds = [CallableSpecKind::InstanceMethod];
        let resolution = class_callable_resolution(
            "queue",
            "PriorityQueue",
            "get",
            &method_kinds,
        )
        .expect("PriorityQueue.get inherited owner");
        let [step] = resolution.path.as_slice() else {
            panic!("PriorityQueue.get must have one generic base step");
        };
        let child = class_by_id(step.child);
        let owner = class_by_id(resolution.owner);
        assert_eq!(string(child.qualifier), "PriorityQueue");
        assert_eq!(string(owner.qualifier), "Queue");
        let [child_param] = class_type_params(child) else {
            panic!("PriorityQueue must retain its class TypeVar");
        };
        let [owner_param] = class_type_params(owner) else {
            panic!("Queue must retain its class TypeVar");
        };
        assert_ne!(child_param, owner_param);
        let [base_arg] = edges(step.args) else {
            panic!("PriorityQueue base must apply one type argument");
        };
        assert_eq!(node(*base_arg), &TypeSpecNode::TypeParam(*child_param));
    }

    #[test]
    fn generated_manifest_marks_attribute_protocols_incomplete() {
        let (_, class) = class_spec("_typeshed", "DataclassInstance")
            .expect("DataclassInstance spec");
        assert_eq!(class.kind, ClassSpecKind::Protocol);
        assert!(!class.method_only_complete);
    }

    #[test]
    fn generated_manifest_excludes_future_only_classes_from_py312() {
        assert!(class_spec("http.server", "_SSLModule").is_none());
        assert!(MANIFEST.classes.iter().all(|class| {
            (string(class.module), string(class.qualifier))
                != ("http.server", "_SSLModule")
        }));
    }

    #[test]
    fn generated_manifest_resolves_aliased_typevar_constructors_and_bounds() {
        let sig = overloads("ast", "", "parse")
            .find(|sig| {
                params(sig.params).first().is_some_and(|param| {
                    matches!(node(type_use(param.ty).0), TypeSpecNode::TypeParam(_))
                })
            })
            .expect("ast.parse TypeVar overload");
        let TypeSpecNode::TypeParam(param_id) = node(type_use(params(sig.params)[0].ty).0)
        else {
            unreachable!()
        };
        let bound = type_param(*param_id).bound.expect("ast._T bound");
        let TypeSpecNode::Name { module, name, kind } = node(bound) else {
            panic!("ast._T bound must retain nominal AST identity")
        };
        assert_eq!((string(*module), string(*name)), ("ast", "AST"));
        assert_eq!(*kind, TypeNameKind::Nominal);
    }

    #[test]
    fn generated_manifest_preserves_string_literal_values() {
        let sig = overloads("time", "", "get_clock_info")
            .next()
            .expect("time.get_clock_info spec");
        let [param] = params(sig.params) else {
            panic!("time.get_clock_info must have one parameter")
        };
        let TypeSpecNode::Apply { args, .. } = node(type_use(param.ty).0) else {
            panic!("clock name must be a Literal application")
        };
        let values: Vec<_> = edges(*args)
            .iter()
            .map(|value| match node(*value) {
                TypeSpecNode::LiteralStr(value) => string(*value),
                other => panic!("clock Literal member was not a string: {other:?}"),
            })
            .collect();
        assert!(values.contains(&"monotonic"));
        assert!(values.contains(&"perf_counter"));
    }

    #[test]
    fn generated_manifest_resolves_implicit_builtins_after_local_symbols() {
        let strict_errors = overloads("codecs", "", "strict_errors")
            .next()
            .expect("codecs.strict_errors spec");
        let [exception] = params(strict_errors.params) else {
            panic!("codecs.strict_errors must have one parameter")
        };
        let TypeSpecNode::Name { module, name, kind } = node(type_use(exception.ty).0)
        else {
            panic!("UnicodeError must retain builtin nominal identity")
        };
        assert_eq!((string(*module), string(*name)), ("builtins", "UnicodeError"));
        assert_eq!(*kind, TypeNameKind::Nominal);

        let warning = overloads("sqlite3", "Connection", "Warning")
            .next()
            .expect("sqlite3.Connection.Warning property spec");
        let TypeSpecNode::Apply { args, .. } = node(type_use(warning.ret).0) else {
            panic!("sqlite3.Connection.Warning must return a type object")
        };
        let [warning_type] = edges(*args) else {
            panic!("sqlite3.Connection.Warning type object must have one argument")
        };
        let TypeSpecNode::Name { module, name, kind } = node(*warning_type) else {
            panic!("sqlite3.Warning must retain local nominal identity")
        };
        assert_eq!((string(*module), string(*name)), ("sqlite3", "Warning"));
        assert_eq!(*kind, TypeNameKind::Nominal);
    }

    #[test]
    fn generated_manifest_canonicalizes_imported_submodule_qualifiers() {
        let init = overloads(
            "asyncio.subprocess",
            "SubprocessStreamProtocol",
            "__init__",
        )
        .next()
        .expect("SubprocessStreamProtocol.__init__ spec");
        let loop_param = params(init.params)
            .iter()
            .find(|param| string(param.name) == "loop")
            .expect("loop parameter");
        let TypeSpecNode::Name { module, name, kind } = node(type_use(loop_param.ty).0)
        else {
            panic!("event loop must retain nominal imported identity")
        };
        assert_eq!(
            (string(*module), string(*name)),
            ("asyncio.events", "AbstractEventLoop")
        );
        assert_eq!(*kind, TypeNameKind::Nominal);

        let discover = overloads("importlib.metadata", "Distribution", "discover")
            .next()
            .expect("Distribution.discover spec");
        let context = params(discover.params)
            .iter()
            .find(|param| string(param.name) == "context")
            .expect("context parameter");
        let TypeSpecNode::Name { module, name, kind } = node(type_use(context.ty).0)
        else {
            panic!("nested class must retain its local identity")
        };
        assert_eq!(
            (string(*module), string(*name)),
            ("importlib.metadata", "DistributionFinder.Context")
        );
        assert_eq!(*kind, TypeNameKind::Nominal);
    }

    #[test]
    fn generated_manifest_preserves_alias_targets() {
        let root_alias = alias("_typeshed", "OpenTextMode").expect("OpenTextMode alias");
        let TypeSpecNode::Union(members) = node(root_alias.target) else {
            panic!("OpenTextMode must retain its alias union target")
        };
        assert!(edges(*members).iter().all(|member| {
            let TypeSpecNode::Name { module, name, kind } = node(*member) else {
                return false;
            };
            if *kind != TypeNameKind::Alias {
                return false;
            }
            let Some(member_alias) = alias(string(*module), string(*name)) else {
                return false;
            };
            let TypeSpecNode::Apply { args, .. } = node(member_alias.target) else {
                return false;
            };
            edges(*args)
                .iter()
                .all(|value| matches!(node(*value), TypeSpecNode::LiteralStr(_)))
        }));
    }
}

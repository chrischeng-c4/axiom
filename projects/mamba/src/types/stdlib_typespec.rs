//! Lossless, generated typeshed callable contracts.
//!
//! The generated artifact is compact structured data rather than Rust source:
//! parsing hundreds of thousands of generated constant lines made every mamba
//! rebuild pay for typeshed. Strings and source locations are interned, while
//! unsupported annotation nodes remain explicit and distinct from `Any`.

use serde::Deserialize;
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
    assert_eq!(manifest.schema, 1, "unsupported typeshed TypeSpec schema");
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
    MANIFEST.callables.iter().filter(move |sig| {
        sig.py312
            && string(sig.module) == module
            && string(sig.qualifier) == qualifier
            && string(sig.name) == name
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

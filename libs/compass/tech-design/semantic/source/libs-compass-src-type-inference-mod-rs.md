---
id: libs-compass-src-type-inference-mod-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/type_inference/mod.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/type_inference/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/type_inference/mod.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `propagation` | libs/compass/src/type_inference/mod.rs | module | pub | 25 | pub mod propagation; |
| `rust_infer` | libs/compass/src/type_inference/mod.rs | module | pub | 34 | pub mod rust_infer; |
| `rust_lifetimes` | libs/compass/src/type_inference/mod.rs | module | pub | 35 | pub mod rust_lifetimes; |
| `rust_symbols` | libs/compass/src/type_inference/mod.rs | module | pub | 36 | pub mod rust_symbols; |
| `rust_traits` | libs/compass/src/type_inference/mod.rs | module | pub | 37 | pub mod rust_traits; |
| `rust_types` | libs/compass/src/type_inference/mod.rs | module | pub | 38 | pub mod rust_types; |
| `ts_infer` | libs/compass/src/type_inference/mod.rs | module | pub | 41 | pub mod ts_infer; |
| `ts_types` | libs/compass/src/type_inference/mod.rs | module | pub | 42 | pub mod ts_types; |
| `refactoring_multilang` | libs/compass/src/type_inference/mod.rs | module | pub | 45 | pub mod refactoring_multilang; |
| `semantic_search_rust` | libs/compass/src/type_inference/mod.rs | module | pub | 46 | pub mod semantic_search_rust; |
| `AnalysisCache` | libs/compass/src/type_inference/mod.rs | re-export | pub | 48 | pub use cache::{AnalysisCache, CacheEntry, ContentHash}; |
| `CacheEntry` | libs/compass/src/type_inference/mod.rs | re-export | pub | 48 | pub use cache::{AnalysisCache, CacheEntry, ContentHash}; |
| `ContentHash` | libs/compass/src/type_inference/mod.rs | re-export | pub | 48 | pub use cache::{AnalysisCache, CacheEntry, ContentHash}; |
| `build_semantic_model` | libs/compass/src/type_inference/mod.rs | re-export | pub | 64 | pub use check::{build_semantic_model, SemanticModelBuilder, TypeChecker, TypeError}; |
| `SemanticModelBuilder` | libs/compass/src/type_inference/mod.rs | re-export | pub | 64 | pub use check::{build_semantic_model, SemanticModelBuilder, TypeChecker, TypeError}; |
| `TypeChecker` | libs/compass/src/type_inference/mod.rs | re-export | pub | 64 | pub use check::{build_semantic_model, SemanticModelBuilder, TypeChecker, TypeError}; |
| `TypeError` | libs/compass/src/type_inference/mod.rs | re-export | pub | 64 | pub use check::{build_semantic_model, SemanticModelBuilder, TypeChecker, TypeError}; |
| `ClassInfo` | libs/compass/src/type_inference/mod.rs | re-export | pub | 65 | pub use class_info::{ClassInfo, GenericParam}; |
| `GenericParam` | libs/compass/src/type_inference/mod.rs | re-export | pub | 65 | pub use class_info::{ClassInfo, GenericParam}; |
| `ArgusConfig` | libs/compass/src/type_inference/mod.rs | re-export | pub | 70 | pub use config::{ArgusConfig, EffectiveConfig, OverrideConfig, PythonEnvConfig}; |
| `EffectiveConfig` | libs/compass/src/type_inference/mod.rs | re-export | pub | 70 | pub use config::{ArgusConfig, EffectiveConfig, OverrideConfig, PythonEnvConfig}; |
| `OverrideConfig` | libs/compass/src/type_inference/mod.rs | re-export | pub | 70 | pub use config::{ArgusConfig, EffectiveConfig, OverrideConfig, PythonEnvConfig}; |
| `PythonEnvConfig` | libs/compass/src/type_inference/mod.rs | re-export | pub | 70 | pub use config::{ArgusConfig, EffectiveConfig, OverrideConfig, PythonEnvConfig}; |
| `TypeInferencer` | libs/compass/src/type_inference/mod.rs | re-export | pub | 93 | pub use infer::{TypeInferencer, TypeVarInfo}; |
| `TypeVarInfo` | libs/compass/src/type_inference/mod.rs | re-export | pub | 93 | pub use infer::{TypeInferencer, TypeVarInfo}; |
| `ModuleGraph` | libs/compass/src/type_inference/mod.rs | re-export | pub | 98 | pub use modules::{ModuleGraph, ModuleNode}; |
| `ModuleNode` | libs/compass/src/type_inference/mod.rs | re-export | pub | 98 | pub use modules::{ModuleGraph, ModuleNode}; |
| `NarrowingCondition` | libs/compass/src/type_inference/mod.rs | re-export | pub | 102 | pub use narrow::{NarrowingCondition, TypeNarrower}; |
| `TypeNarrower` | libs/compass/src/type_inference/mod.rs | re-export | pub | 102 | pub use narrow::{NarrowingCondition, TypeNarrower}; |
| `ProjectAnalyzer` | libs/compass/src/type_inference/mod.rs | re-export | pub | 106 | pub use project::{ProjectAnalyzer, ProjectConfig}; |
| `ProjectConfig` | libs/compass/src/type_inference/mod.rs | re-export | pub | 106 | pub use project::{ProjectAnalyzer, ProjectConfig}; |
| `StubLoader` | libs/compass/src/type_inference/mod.rs | re-export | pub | 116 | pub use stubs::StubLoader; |
| `LiteralValue` | libs/compass/src/type_inference/mod.rs | re-export | pub | 117 | pub use ty::{LiteralValue, Param, ParamKind, Type, TypeVarId, Variance}; |
| `Param` | libs/compass/src/type_inference/mod.rs | re-export | pub | 117 | pub use ty::{LiteralValue, Param, ParamKind, Type, TypeVarId, Variance}; |
| `ParamKind` | libs/compass/src/type_inference/mod.rs | re-export | pub | 117 | pub use ty::{LiteralValue, Param, ParamKind, Type, TypeVarId, Variance}; |
| `Type` | libs/compass/src/type_inference/mod.rs | re-export | pub | 117 | pub use ty::{LiteralValue, Param, ParamKind, Type, TypeVarId, Variance}; |
| `TypeVarId` | libs/compass/src/type_inference/mod.rs | re-export | pub | 117 | pub use ty::{LiteralValue, Param, ParamKind, Type, TypeVarId, Variance}; |
| `Variance` | libs/compass/src/type_inference/mod.rs | re-export | pub | 117 | pub use ty::{LiteralValue, Param, ParamKind, Type, TypeVarId, Variance}; |
| `TypeEnv` | libs/compass/src/type_inference/mod.rs | re-export | pub | 118 | pub use type_env::TypeEnv; |
| `TypeshedCache` | libs/compass/src/type_inference/mod.rs | re-export | pub | 119 | pub use typeshed::{TypeshedCache, TypeshedConfig}; |
| `TypeshedConfig` | libs/compass/src/type_inference/mod.rs | re-export | pub | 119 | pub use typeshed::{TypeshedCache, TypeshedConfig}; |
| `rust_advanced` | libs/compass/src/type_inference/mod.rs | module | pub | 122 | pub mod rust_advanced; |
| `ts_advanced` | libs/compass/src/type_inference/mod.rs | module | pub | 123 | pub mod ts_advanced; |
| `TsTypeInferencer` | libs/compass/src/type_inference/mod.rs | re-export | pub | 129 | pub use ts_infer::TsTypeInferencer; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Type system for Argus
//!
//! This module provides type inference and checking for Python, TypeScript, and Rust.

mod annotation;
mod builtins;
mod cache;
mod cfg_narrow;
mod check;
mod class_info;
mod codegen;
mod config;
mod deep_inference;
mod env;
mod frameworks;
mod imports;
mod incremental;
mod infer;
mod model;
mod modules;
mod mutable_ast;
mod narrow;
mod package_managers;
mod project;
pub mod propagation;
mod refactoring;
mod semantic_search;
mod stubs;
mod ty;
mod type_env;
mod typeshed;

// Rust-specific type system extensions
pub mod rust_infer;
pub mod rust_lifetimes;
pub mod rust_symbols;
pub mod rust_traits;
pub mod rust_types;

// TypeScript-specific type system extensions
pub mod ts_infer;
pub mod ts_types;

// Unified multi-language extensions
pub mod refactoring_multilang;
pub mod semantic_search_rust;

pub use cache::{AnalysisCache, CacheEntry, ContentHash};
pub use cfg_narrow::{
    apply_typevar_bindings,
    // Protocol structural typing (R2.3)
    check_protocol_satisfaction,
    // Overload resolution (R2.4)
    resolve_overload,
    // TypeVar resolution (R2.2)
    resolve_typevar_bindings,
    // CFG-based narrowing (R2.1)
    BlockNarrowEnv,
    CfgNarrowingPass,
    CfgNarrowingResult,
    ProtocolCheckResult,
    ProtocolMemberError,
};
pub use check::{build_semantic_model, SemanticModelBuilder, TypeChecker, TypeError};
pub use class_info::{ClassInfo, GenericParam};
pub use codegen::{
    CodeGenKind, CodeGenOptions, CodeGenRequest, CodeGenResult, CodeGenerator, DocstringStyle,
    TestFramework,
};
pub use config::{ArgusConfig, EffectiveConfig, OverrideConfig, PythonEnvConfig};
pub use deep_inference::{
    infer_type_deep, trace_type_chain, CrossFileRef, DeepInferenceResult, DeepTypeInferencer,
    FileAnalysis, GenericKey, ImportGraph, ImportInfo, MethodSignature, ProtocolDef, TypeBinding,
    TypeContext, TypeTraceStep, TypeVarInfo as DeepTypeVarInfo,
};
pub use env::{
    detect_all_venvs, detect_python_environment, detect_with_config, find_site_packages,
    get_venv_python_version, is_venv_directory, DetectedEnv, EnvInfo, VenvType,
};
pub use frameworks::{
    DjangoField, DjangoFieldType, DjangoModel, DjangoRelation, DjangoRelationType,
    DjangoTypeProvider, FastAPIEndpoint, FastAPITypeProvider, Framework, FrameworkDetection,
    FrameworkDetector, FrameworkRegistry, FrameworkTypeProvider, MethodType, PydanticConfig,
    PydanticExtra, PydanticField, PydanticModel, PydanticTypeProvider,
};
pub use imports::{
    Import, ImportResolver, ImportedName, ModuleIndexEntry, ModuleInfo, ModuleLoadState,
};
pub use incremental::{
    AnalysisResult, CachedAnalysis, ChangeKind, ChangeTracker, DependencyGraph, FileChange,
    IncrementalAnalyzer, IncrementalConfig,
};
pub use infer::{TypeInferencer, TypeVarInfo};
pub use model::{
    LiteralInfo, ParamInfo, ScopeId, ScopeInfo, SemanticModel, SemanticSymbolKind, SymbolData,
    SymbolId, SymbolReference, TypeInfo, TypedRange,
};
pub use modules::{ModuleGraph, ModuleNode};
pub use mutable_ast::{
    AstEdit, MutableAst, MutableNode, NodeId, NodeMetadata, NodeRef, Span, TreeDiff,
};
pub use narrow::{NarrowingCondition, TypeNarrower};
pub use package_managers::{
    Dependency, PackageManager, PackageManagerDetection, PackageManagerDetector,
};
pub use project::{ProjectAnalyzer, ProjectConfig};
pub use refactoring::{
    DiagnosticLevel, ImportChange, RefactorDiagnostic, RefactorKind, RefactorOptions,
    RefactorRequest, RefactorResult, RefactoringEngine, SignatureChanges, TextEdit,
};
pub use semantic_search::{
    CallDirection, MatchContext, MatchKind, SearchKind, SearchMatch, SearchQuery, SearchResult,
    SearchScope, SearchStats, SemanticSearchEngine, SymbolLocation, TypeHierarchyDirection,
    TypeLocation,
};
pub use stubs::StubLoader;
pub use ty::{LiteralValue, Param, ParamKind, Type, TypeVarId, Variance};
pub use type_env::TypeEnv;
pub use typeshed::{TypeshedCache, TypeshedConfig};

// Advanced type inference modules (R1, R2)
pub mod rust_advanced;
pub mod ts_advanced;

// TypeScript type system exports
pub use propagation::{
    PropagatedType, PropagationPipeline, PropagationRequest, PropagationResult, PropagationStats,
};
pub use ts_infer::TsTypeInferencer;
pub use ts_types::{
    is_assignable_to, MappedTypeModifier, TemplatePart, TsClass, TsConditionalType, TsEnum,
    TsEnumValue, TsInterface, TsMappedType, TsProperty, TsTemplateLiteralType, TsTypeAlias,
    TsTypeContext, TsTypeParam, Visibility,
};
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/type_inference/mod.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/type_inference/mod.rs` captured during libs codegen standardization.
```

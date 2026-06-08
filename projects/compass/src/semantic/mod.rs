//! Advanced code analysis

pub mod pdg;
pub mod scope;
pub mod symbols;
pub mod types;

#[cfg(test)]
mod tests;

pub use pdg::{
    detect_taint_sinks,
    detect_taint_sources,
    CfgBuilder,
    ControlDependencies,
    ControlFlowGraph,
    DataDependencies,
    DeadCodeAnalysis,
    DependencyReason,
    DetectedSink,
    DetectedSource,
    ImpactAnalysis,
    // Dependency tree (R6)
    ImpactAnalysisTree,
    ImpactTreeNode,
    PdgEdge,
    PdgEdgeJson,
    PdgEdgeKind,
    PdgJson,
    PdgNode,
    PdgNodeJson,
    ProgramDependenceGraph,
    ProgramSlice,
    // Semantic taint (R7)
    SemanticTaintAnalysis,
    SliceDirection,
    TaintAnalysis,
    TaintSinkKind,
    TaintSourceKind,
};
pub use scope::{Scope, ScopeAnalyzer, ScopeKind};
pub use scope::{Symbol as ScopeSymbol, SymbolKind as ScopeSymbolKind};
pub use symbols::{
    Symbol, SymbolId, SymbolKind, SymbolReference, SymbolTable, SymbolTableBuilder, TypeInfo,
};

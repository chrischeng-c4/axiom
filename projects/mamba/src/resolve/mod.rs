pub mod pass;
pub mod scope;

pub use pass::{resolve_module, ResolveResult};
pub use scope::{SymbolId, SymbolInfo, SymbolKind, SymbolTable, VariableClass};

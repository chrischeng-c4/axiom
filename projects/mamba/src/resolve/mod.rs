pub mod scope;
pub mod pass;

pub use scope::{SymbolId, SymbolInfo, SymbolKind, SymbolTable, VariableClass};
pub use pass::{resolve_module, ResolveResult};

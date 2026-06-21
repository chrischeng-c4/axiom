use std::collections::HashMap;

/// Unique identifier for a symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);

/// Information about a declared symbol.
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable,
    Function,
    Parameter,
    Class,
    Enum,
    EnumVariant,
    Module,
}

/// Classification of how a variable is accessed at runtime.
/// Determined by the resolver after processing global/nonlocal declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableClass {
    /// Stored in the function's local variable slots.
    Local,
    /// Accessed from the module-level namespace (declared `global`).
    Global,
    /// Referenced in current scope but bound in an enclosing function.
    Free,
    /// A local variable that is also captured by an inner scope as free/nonlocal.
    Cell,
}

/// A scope containing symbol bindings.
#[derive(Debug)]
pub struct Scope {
    pub parent: Option<usize>,
    symbols: HashMap<String, SymbolId>,
}

impl Scope {
    pub fn new(parent: Option<usize>) -> Self {
        Self {
            parent,
            symbols: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, id: SymbolId) {
        self.symbols.insert(name, id);
    }

    pub fn lookup(&self, name: &str) -> Option<SymbolId> {
        self.symbols.get(name).copied()
    }
}

/// Symbol table managing all scopes and symbols.
#[derive(Debug)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
    symbols: Vec<SymbolInfo>,
    current_scope: usize,
    /// Variable classification for symbols (populated by resolver).
    var_classes: HashMap<SymbolId, VariableClass>,
    /// Maps inner (Free) SymbolId → outer (Cell) SymbolId for nonlocal variables.
    nonlocal_mapping: HashMap<SymbolId, SymbolId>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let global = Scope::new(None);
        Self {
            scopes: vec![global],
            symbols: Vec::new(),
            current_scope: 0,
            var_classes: HashMap::new(),
            nonlocal_mapping: HashMap::new(),
        }
    }

    pub fn push_scope(&mut self) {
        let parent = self.current_scope;
        let scope = Scope::new(Some(parent));
        self.current_scope = self.scopes.len();
        self.scopes.push(scope);
    }

    pub fn pop_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current_scope].parent {
            self.current_scope = parent;
        }
    }

    pub fn define(&mut self, name: String, kind: SymbolKind) -> SymbolId {
        let id = SymbolId(self.symbols.len() as u32);
        self.symbols.push(SymbolInfo {
            id,
            name: name.clone(),
            kind,
        });
        self.scopes[self.current_scope].define(name, id);
        id
    }

    pub fn lookup(&self, name: &str) -> Option<SymbolId> {
        let mut scope_idx = self.current_scope;
        loop {
            if let Some(id) = self.scopes[scope_idx].lookup(name) {
                return Some(id);
            }
            match self.scopes[scope_idx].parent {
                Some(parent) => scope_idx = parent,
                None => return None,
            }
        }
    }

    pub fn get_symbol(&self, id: SymbolId) -> &SymbolInfo {
        &self.symbols[id.0 as usize]
    }

    /// Return all defined symbols (#1190).
    /// Used by module import to build SymbolId → name mapping.
    pub fn all_symbols(&self) -> &[SymbolInfo] {
        &self.symbols
    }

    /// Set the variable classification for a symbol.
    pub fn set_var_class(&mut self, id: SymbolId, class: VariableClass) {
        self.var_classes.insert(id, class);
    }

    /// Get the variable classification for a symbol.
    /// Returns `Local` by default if not explicitly classified.
    pub fn get_var_class(&self, id: SymbolId) -> VariableClass {
        self.var_classes
            .get(&id)
            .copied()
            .unwrap_or(VariableClass::Local)
    }

    /// Get the current scope index.
    pub fn current_scope_idx(&self) -> usize {
        self.current_scope
    }

    /// Get the parent scope index for a given scope.
    pub fn parent_scope(&self, scope_idx: usize) -> Option<usize> {
        self.scopes[scope_idx].parent
    }

    /// Look up a name in a specific scope (not walking parents).
    pub fn lookup_in_scope(&self, scope_idx: usize, name: &str) -> Option<SymbolId> {
        self.scopes[scope_idx].lookup(name)
    }

    /// Define a symbol in the enclosing (parent) scope.
    /// PEP 572: walrus `:=` inside a comprehension defines in the enclosing scope.
    /// Falls back to current scope if there is no parent.
    pub fn define_in_enclosing_scope(&mut self, name: String, kind: SymbolKind) -> SymbolId {
        let target_scope = self.scopes[self.current_scope]
            .parent
            .unwrap_or(self.current_scope);
        self.define_in_scope(target_scope, name, kind)
    }

    /// Define a name `levels` parent-scopes up from the current scope. For a
    /// walrus inside NESTED comprehensions (PEP 572) the target must bind in the
    /// nearest enclosing NON-comprehension scope — i.e. skip past each
    /// comprehension's own pushed scope (`comprehension_depth` of them). With
    /// `levels == 1` this matches define_in_enclosing_scope (single comp).
    pub fn define_levels_up(&mut self, levels: usize, name: String, kind: SymbolKind) -> SymbolId {
        let mut scope = self.current_scope;
        for _ in 0..levels {
            scope = self.scopes[scope].parent.unwrap_or(scope);
        }
        self.define_in_scope(scope, name, kind)
    }

    /// Define a symbol in a specific scope (for walrus-in-comprehension, PEP 572).
    pub fn define_in_scope(
        &mut self,
        scope_idx: usize,
        name: String,
        kind: SymbolKind,
    ) -> SymbolId {
        let id = SymbolId(self.symbols.len() as u32);
        self.symbols.push(SymbolInfo {
            id,
            name: name.clone(),
            kind,
        });
        self.scopes[scope_idx].define(name, id);
        id
    }

    /// Record that an inner (Free) symbol refers to an outer (Cell) symbol.
    pub fn set_nonlocal_mapping(&mut self, inner: SymbolId, outer: SymbolId) {
        self.nonlocal_mapping.insert(inner, outer);
    }

    /// Get the outer (Cell) SymbolId for an inner (Free) symbol, if any.
    pub fn get_nonlocal_outer(&self, inner: SymbolId) -> Option<SymbolId> {
        self.nonlocal_mapping.get(&inner).copied()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: tick-128 test-coverage — SymbolTable scope stack, var classification, nonlocal mapping
    #[test]
    fn test_symbol_table_scope_stack_and_classifications() {
        let mut st = SymbolTable::new();
        assert_eq!(st.current_scope_idx(), 0);

        // global define + lookup
        let g = st.define("x".into(), SymbolKind::Variable);
        assert_eq!(st.lookup("x"), Some(g));
        assert_eq!(st.get_symbol(g).name, "x");
        assert_eq!(st.all_symbols().len(), 1);

        // push_scope: inner shadow define
        st.push_scope();
        let inner = st.current_scope_idx();
        assert_eq!(st.parent_scope(inner), Some(0));
        let s = g; // outer id captured for later
        let inner_x = st.define("x".into(), SymbolKind::Parameter);
        assert_ne!(inner_x, s);
        assert_eq!(st.lookup("x"), Some(inner_x)); // walks up → inner wins
        assert_eq!(st.lookup_in_scope(0, "x"), Some(s)); // no-walk → outer
        assert_eq!(st.lookup_in_scope(inner, "x"), Some(inner_x));

        // pop_scope returns to global — inner shadow gone
        st.pop_scope();
        assert_eq!(st.current_scope_idx(), 0);
        assert_eq!(st.lookup("x"), Some(s));

        // var_class defaults to Local, then set+get round-trip
        assert_eq!(st.get_var_class(s), VariableClass::Local);
        st.set_var_class(s, VariableClass::Cell);
        assert_eq!(st.get_var_class(s), VariableClass::Cell);

        // nonlocal mapping: inner Free → outer Cell
        st.push_scope();
        let f = st.define("y".into(), SymbolKind::Variable);
        st.set_nonlocal_mapping(f, s);
        assert_eq!(st.get_nonlocal_outer(f), Some(s));
        assert_eq!(st.get_nonlocal_outer(s), None);

        // define_in_enclosing_scope: walrus PEP 572 — defines in parent (scope 0)
        let w = st.define_in_enclosing_scope("w".into(), SymbolKind::Variable);
        assert_eq!(st.lookup_in_scope(0, "w"), Some(w));
        assert_eq!(st.lookup_in_scope(st.current_scope_idx(), "w"), None);
    }

    // REQ: tick-140 test-coverage — SymbolTable edge cases: root pop is no-op, lookup miss returns None, define_in_scope writes to explicit scope.
    #[test]
    fn test_symbol_table_edge_cases_pop_root_lookup_miss_define_in_explicit_scope() {
        let mut st = SymbolTable::new();
        st.pop_scope();
        assert_eq!(st.current_scope_idx(), 0);
        assert_eq!(st.lookup("nonexistent"), None);
        assert_eq!(st.lookup_in_scope(0, "nonexistent"), None);
        st.push_scope();
        let inner = st.current_scope_idx();
        let g = st.define_in_scope(0, "glob".into(), SymbolKind::Function);
        assert_eq!(st.lookup_in_scope(0, "glob"), Some(g));
        assert_eq!(st.lookup_in_scope(inner, "glob"), None);
        assert_eq!(st.lookup("glob"), Some(g));
    }
}

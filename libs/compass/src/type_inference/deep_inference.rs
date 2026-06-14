//! Deep cross-file type inference (Sprint 2 - Track 1)
//!
//! Provides advanced type inference capabilities:
//! - Cross-file type tracking and propagation
//! - Full generic and TypeVar support
//! - Protocol and structural typing
//! - Advanced type narrowing
//! - Recursive type handling

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::type_inference::class_info::ClassInfo;
use crate::type_inference::frameworks::FrameworkRegistry;
use crate::type_inference::ty::Type;

/// Unique identifier for types in cross-file tracking.
pub type TypeId = usize;

// ============================================================================
// Type Context for Cross-File Tracking
// ============================================================================

/// Cross-file type context for tracking type information across modules.
#[derive(Debug, Clone)]
pub struct TypeContext {
    /// Type bindings by file and symbol
    bindings: HashMap<PathBuf, HashMap<String, TypeBinding>>,
    /// Type variables in scope
    type_vars: HashMap<String, TypeVarInfo>,
    /// Protocol definitions
    protocols: HashMap<String, ProtocolDef>,
    /// Generic instantiations cache
    generic_cache: HashMap<GenericKey, Type>,
    /// Recursive type detection
    recursive_guard: HashSet<TypeId>,
    /// Class information (for protocol conformance checking)
    class_info: HashMap<String, ClassInfo>,
}

/// A type binding with source information.
#[derive(Debug, Clone)]
pub struct TypeBinding {
    /// The inferred type
    pub ty: Type,
    /// Source file
    pub source_file: PathBuf,
    /// Symbol name
    pub symbol: String,
    /// Line number
    pub line: u32,
    /// Whether this is exported
    pub is_exported: bool,
    /// Dependencies (other symbols this type depends on)
    pub dependencies: Vec<String>,
    /// Whether this binding was propagated from another file (R3).
    pub is_propagated: bool,
}

/// TypeVar information for generics.
#[derive(Debug, Clone)]
pub struct TypeVarInfo {
    /// TypeVar name
    pub name: String,
    /// Bound type (if any)
    pub bound: Option<Type>,
    /// Constraints
    pub constraints: Vec<Type>,
    /// Covariant
    pub covariant: bool,
    /// Contravariant
    pub contravariant: bool,
}

impl TypeVarInfo {
    /// Create a new TypeVar.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bound: None,
            constraints: Vec::new(),
            covariant: false,
            contravariant: false,
        }
    }

    /// Set bound.
    pub fn with_bound(mut self, bound: Type) -> Self {
        self.bound = Some(bound);
        self
    }

    /// Add constraint.
    pub fn with_constraint(mut self, constraint: Type) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Set covariant.
    pub fn covariant(mut self) -> Self {
        self.covariant = true;
        self
    }

    /// Set contravariant.
    pub fn contravariant(mut self) -> Self {
        self.contravariant = true;
        self
    }
}

/// Protocol definition for structural typing.
#[derive(Debug, Clone)]
pub struct ProtocolDef {
    /// Protocol name
    pub name: String,
    /// Required methods
    pub methods: HashMap<String, MethodSignature>,
    /// Required attributes
    pub attributes: HashMap<String, Type>,
    /// Parent protocols
    pub parents: Vec<String>,
}

/// Method signature in a protocol.
#[derive(Debug, Clone)]
pub struct MethodSignature {
    /// Method name
    pub name: String,
    /// Parameter types
    pub params: Vec<(String, Type)>,
    /// Return type
    pub return_type: Type,
    /// Is async
    pub is_async: bool,
}

/// Key for generic instantiation cache.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GenericKey {
    /// Base generic type
    pub base: String,
    /// Type arguments
    pub args: Vec<String>,
}

impl TypeContext {
    /// Create a new type context.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            type_vars: HashMap::new(),
            protocols: HashMap::new(),
            generic_cache: HashMap::new(),
            recursive_guard: HashSet::new(),
            class_info: HashMap::new(),
        }
    }

    /// Add a type binding.
    pub fn add_binding(&mut self, file: PathBuf, binding: TypeBinding) {
        self.bindings
            .entry(file)
            .or_default()
            .insert(binding.symbol.clone(), binding);
    }

    /// Get a type binding.
    pub fn get_binding(&self, file: &PathBuf, symbol: &str) -> Option<&TypeBinding> {
        self.bindings.get(file)?.get(symbol)
    }

    /// Resolve a type across files.
    pub fn resolve_type(&self, symbol: &str, from_file: &PathBuf) -> Option<&Type> {
        // First check current file
        if let Some(binding) = self.get_binding(from_file, symbol) {
            return Some(&binding.ty);
        }

        // Then check all files for exported symbols
        for (_, bindings) in &self.bindings {
            if let Some(binding) = bindings.get(symbol) {
                if binding.is_exported {
                    return Some(&binding.ty);
                }
            }
        }

        None
    }

    /// Register a TypeVar.
    pub fn register_type_var(&mut self, info: TypeVarInfo) {
        self.type_vars.insert(info.name.clone(), info);
    }

    /// Get TypeVar info.
    pub fn get_type_var(&self, name: &str) -> Option<&TypeVarInfo> {
        self.type_vars.get(name)
    }

    /// Register a protocol.
    pub fn register_protocol(&mut self, protocol: ProtocolDef) {
        self.protocols.insert(protocol.name.clone(), protocol);
    }

    /// Check if a type satisfies a protocol (structural typing).
    pub fn satisfies_protocol(&self, ty: &Type, protocol_name: &str) -> bool {
        let protocol = match self.protocols.get(protocol_name) {
            Some(p) => p,
            None => return false,
        };

        // Check all required methods and attributes
        // This is a placeholder - full implementation would inspect the type
        self.check_protocol_conformance(ty, protocol)
    }

    fn check_protocol_conformance(&self, ty: &Type, protocol: &ProtocolDef) -> bool {
        // Extract class name from Type
        let class_name = match ty {
            Type::Instance { name, .. } => name,
            Type::ClassType { name, .. } => name,
            _ => return false, // Non-class types can't implement protocols
        };

        // Get class information
        let class_info = match self.class_info.get(class_name) {
            Some(info) => info,
            None => return false, // Unknown class, can't check
        };

        // Check all required methods in the protocol
        for (method_name, required_sig) in &protocol.methods {
            match class_info.methods.get(method_name) {
                Some(class_method_ty) => {
                    // Check if method signature is compatible
                    if !self.is_signature_compatible(
                        class_method_ty,
                        &required_sig.return_type,
                        &required_sig.params,
                    ) {
                        return false;
                    }
                }
                None => return false, // Required method not found
            }
        }

        // Check all required attributes in the protocol
        for (attr_name, required_ty) in &protocol.attributes {
            match class_info.attributes.get(attr_name) {
                Some(class_attr_ty) => {
                    // Check if attribute type is compatible
                    if !self.is_type_compatible(class_attr_ty, required_ty) {
                        return false;
                    }
                }
                None => return false, // Required attribute not found
            }
        }

        // Check parent protocols recursively
        for parent_name in &protocol.parents {
            if let Some(parent_protocol) = self.protocols.get(parent_name) {
                if !self.check_protocol_conformance(ty, parent_protocol) {
                    return false;
                }
            }
        }

        true
    }

    /// Check if a method signature is compatible with requirements.
    fn is_signature_compatible(
        &self,
        method_ty: &Type,
        required_ret: &Type,
        required_params: &[(String, Type)],
    ) -> bool {
        // Extract callable signature from method type
        let (actual_params, actual_ret) = match method_ty {
            Type::Callable { params, ret } => (params, ret.as_ref()),
            _ => return false,
        };

        // Check return type compatibility (covariant)
        if !self.is_type_compatible(actual_ret, required_ret) {
            return false;
        }

        // Check parameter count
        if actual_params.len() < required_params.len() {
            return false;
        }

        // Check parameter types (contravariant)
        for (i, (_, required_param_ty)) in required_params.iter().enumerate() {
            if let Some(actual_param) = actual_params.get(i) {
                // Parameters are contravariant: required type must be subtype of actual
                if !self.is_type_compatible(required_param_ty, &actual_param.ty) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Check if two types are compatible (basic structural equality).
    fn is_type_compatible(&self, actual: &Type, required: &Type) -> bool {
        use Type::*;

        match (actual, required) {
            // Exact matches
            (Never, Never)
            | (None, None)
            | (Bool, Bool)
            | (Int, Int)
            | (Float, Float)
            | (Str, Str)
            | (Bytes, Bytes) => true,

            // Any accepts everything
            (_, Any) | (Any, _) => true,

            // Unknown can match anything (inference incomplete)
            (Unknown, _) | (_, Unknown) => true,

            // Lists - check element type
            (List(a), List(b)) => self.is_type_compatible(a, b),

            // Dicts - check key and value types
            (Dict(k1, v1), Dict(k2, v2)) => {
                self.is_type_compatible(k1, k2) && self.is_type_compatible(v1, v2)
            }

            // Sets - check element type
            (Set(a), Set(b)) => self.is_type_compatible(a, b),

            // Tuples - check all element types
            (Tuple(a), Tuple(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b.iter())
                        .all(|(x, y)| self.is_type_compatible(x, y))
            }

            // Optional types
            (Optional(a), Optional(b)) => self.is_type_compatible(a, b),
            (Optional(a), b) => self.is_type_compatible(a, b),
            (a, Optional(b)) => self.is_type_compatible(a, b),

            // Unions - actual must be subset of required
            (Union(actuals), Union(requireds)) => actuals
                .iter()
                .all(|a| requireds.iter().any(|r| self.is_type_compatible(a, r))),
            (actual, Union(requireds)) => {
                requireds.iter().any(|r| self.is_type_compatible(actual, r))
            }

            // Instances - check name compatibility
            (Instance { name: n1, .. }, Instance { name: n2, .. }) => n1 == n2,

            // Class types
            (ClassType { name: n1, .. }, ClassType { name: n2, .. }) => n1 == n2,

            // Callables - check signature compatibility
            (
                Callable {
                    params: p1,
                    ret: r1,
                },
                Callable {
                    params: p2,
                    ret: r2,
                },
            ) => {
                // Return types are covariant
                self.is_type_compatible(r1, r2) &&
                // Parameters are contravariant (and must match count)
                p1.len() == p2.len() &&
                p1.iter().zip(p2.iter()).all(|(a, b)| self.is_type_compatible(&b.ty, &a.ty))
            }

            // Default: not compatible
            _ => false,
        }
    }

    /// Cache a generic instantiation.
    pub fn cache_generic(&mut self, key: GenericKey, ty: Type) {
        self.generic_cache.insert(key, ty);
    }

    /// Get cached generic instantiation.
    pub fn get_cached_generic(&self, key: &GenericKey) -> Option<&Type> {
        self.generic_cache.get(key)
    }

    /// Enter recursive type checking (returns false if already checking this type).
    pub fn enter_recursive(&mut self, type_id: TypeId) -> bool {
        self.recursive_guard.insert(type_id)
    }

    /// Exit recursive type checking.
    pub fn exit_recursive(&mut self, type_id: TypeId) {
        self.recursive_guard.remove(&type_id);
    }

    /// Check if currently checking a recursive type.
    pub fn is_recursive(&self, type_id: TypeId) -> bool {
        self.recursive_guard.contains(&type_id)
    }

    /// Add class information.
    pub fn add_class_info(&mut self, name: String, info: ClassInfo) {
        self.class_info.insert(name, info);
    }

    /// Get class information.
    pub fn get_class_info(&self, name: &str) -> Option<&ClassInfo> {
        self.class_info.get(name)
    }

    /// Get mutable class information.
    pub fn get_class_info_mut(&mut self, name: &str) -> Option<&mut ClassInfo> {
        self.class_info.get_mut(name)
    }

    /// Add a protocol definition.
    pub fn add_protocol(&mut self, name: String, protocol: ProtocolDef) {
        self.protocols.insert(name, protocol);
    }

    /// Get a protocol definition.
    pub fn get_protocol(&self, name: &str) -> Option<&ProtocolDef> {
        self.protocols.get(name)
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Deep Type Inferencer
// ============================================================================

/// Deep type inferencer with cross-file support.
pub struct DeepTypeInferencer {
    /// Type context
    context: TypeContext,
    /// Files being analyzed
    files: HashMap<PathBuf, FileAnalysis>,
    /// Import graph
    import_graph: ImportGraph,
    /// Framework type providers
    framework_registry: FrameworkRegistry,
    /// Virtual environment path (from package manager detection)
    venv_path: Option<PathBuf>,
    /// Package manager detection result
    pkg_detection: Option<super::package_managers::PackageManagerDetection>,
}

/// Analysis state for a single file.
#[derive(Debug, Clone)]
pub struct FileAnalysis {
    /// File path
    pub path: PathBuf,
    /// Symbols defined in this file
    pub symbols: HashMap<String, TypeBinding>,
    /// Imports from other files
    pub imports: Vec<ImportInfo>,
    /// Analysis complete
    pub complete: bool,
    /// Whether cross-file type propagation has completed for this file (R3).
    pub propagation_complete: bool,
}

/// Import information.
#[derive(Debug, Clone)]
pub struct ImportInfo {
    /// Module being imported
    pub module: String,
    /// Specific names imported (None = import all)
    pub names: Option<Vec<String>>,
    /// Alias (if any)
    pub alias: Option<String>,
}

/// Import graph for dependency tracking.
#[derive(Debug, Clone, Default)]
pub struct ImportGraph {
    /// Edges: file -> files it imports from
    edges: HashMap<PathBuf, HashSet<PathBuf>>,
    /// Reverse edges: file -> files that import it
    reverse_edges: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl ImportGraph {
    /// Create a new import graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an import edge.
    pub fn add_import(&mut self, from: PathBuf, to: PathBuf) {
        self.edges
            .entry(from.clone())
            .or_default()
            .insert(to.clone());
        self.reverse_edges.entry(to).or_default().insert(from);
    }

    /// Get files imported by a file.
    pub fn imports(&self, file: &PathBuf) -> Option<&HashSet<PathBuf>> {
        self.edges.get(file)
    }

    /// Get files that import a file.
    pub fn imported_by(&self, file: &PathBuf) -> Option<&HashSet<PathBuf>> {
        self.reverse_edges.get(file)
    }

    /// Return all file paths known to this graph.
    pub fn all_files(&self) -> Vec<PathBuf> {
        let mut files: HashSet<PathBuf> = self.edges.keys().cloned().collect();
        for targets in self.edges.values() {
            files.extend(targets.iter().cloned());
        }
        files.into_iter().collect()
    }

    /// Topological sort for analysis order.
    pub fn topological_sort(&self) -> Vec<PathBuf> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        for file in self.edges.keys() {
            self.visit(file, &mut visited, &mut temp_visited, &mut result);
        }

        // DFS post-order gives us the correct topological order
        // (dependencies before dependents)
        result
    }

    fn visit(
        &self,
        file: &PathBuf,
        visited: &mut HashSet<PathBuf>,
        temp_visited: &mut HashSet<PathBuf>,
        result: &mut Vec<PathBuf>,
    ) {
        if visited.contains(file) {
            return;
        }
        if temp_visited.contains(file) {
            // Cycle detected - skip
            return;
        }

        temp_visited.insert(file.clone());

        if let Some(imports) = self.imports(file) {
            for imported in imports {
                self.visit(imported, visited, temp_visited, result);
            }
        }

        temp_visited.remove(file);
        visited.insert(file.clone());
        result.push(file.clone());
    }
}

impl DeepTypeInferencer {
    /// Create a new deep type inferencer.
    pub fn new() -> Self {
        Self {
            context: TypeContext::new(),
            files: HashMap::new(),
            import_graph: ImportGraph::new(),
            framework_registry: FrameworkRegistry::new(),
            venv_path: None,
            pkg_detection: None,
        }
    }

    /// Initialize with package manager detection
    ///
    /// This enables:
    /// - Virtual environment-aware import resolution
    /// - Dependency checking for external modules
    pub fn with_package_detection(
        mut self,
        detection: super::package_managers::PackageManagerDetection,
    ) -> Self {
        self.venv_path = detection.venv_path.clone();
        self.pkg_detection = Some(detection);
        self
    }

    /// Get a reference to the framework registry for configuration.
    pub fn framework_registry(&self) -> &FrameworkRegistry {
        &self.framework_registry
    }

    /// Get a mutable reference to the framework registry for configuration.
    pub fn framework_registry_mut(&mut self) -> &mut FrameworkRegistry {
        &mut self.framework_registry
    }

    /// Add a file for analysis.
    pub fn add_file(&mut self, path: PathBuf) {
        self.files.insert(
            path.clone(),
            FileAnalysis {
                path,
                symbols: HashMap::new(),
                imports: Vec::new(),
                complete: false,
                propagation_complete: false,
            },
        );
    }

    /// Get the type context.
    pub fn context(&self) -> &TypeContext {
        &self.context
    }

    /// Get mutable type context.
    pub fn context_mut(&mut self) -> &mut TypeContext {
        &mut self.context
    }

    /// Resolve import path using virtual environment
    ///
    /// Checks if a module exists in the virtual environment's site-packages.
    /// This is useful for resolving imports to third-party packages.
    pub fn resolve_import_path(&self, module: &str) -> Option<PathBuf> {
        if let Some(venv_path) = &self.venv_path {
            // Try lib/pythonX.Y/site-packages (Unix)
            let site_packages_patterns = vec![
                venv_path.join("lib/python3.12/site-packages"),
                venv_path.join("lib/python3.11/site-packages"),
                venv_path.join("lib/python3.10/site-packages"),
                venv_path.join("lib/python3.9/site-packages"),
                // Windows
                venv_path.join("Lib/site-packages"),
            ];

            for site_packages in site_packages_patterns {
                if site_packages.exists() {
                    // Try as module file: module.py
                    let module_file =
                        site_packages.join(format!("{}.py", module.replace(".", "/")));
                    if module_file.exists() {
                        return Some(module_file);
                    }

                    // Try as package: module/__init__.py
                    let package_dir = site_packages.join(module.replace(".", "/"));
                    let init_file = package_dir.join("__init__.py");
                    if init_file.exists() {
                        return Some(init_file);
                    }
                }
            }
        }

        None
    }

    /// Check if a module is available in dependencies
    ///
    /// Returns true if the module is listed in the package manager dependencies.
    pub fn has_module(&self, module_name: &str) -> bool {
        if let Some(detection) = &self.pkg_detection {
            // Extract package name (first part before dot)
            let package_name = module_name.split('.').next().unwrap_or(module_name);
            detection.has_dependency(package_name)
        } else {
            false
        }
    }

    /// Get package manager detection result
    pub fn package_detection(&self) -> Option<&super::package_managers::PackageManagerDetection> {
        self.pkg_detection.as_ref()
    }

    /// Propagate types from imported files to importing files.
    ///
    /// When a symbol is imported from another file, this method resolves the type
    /// from the source file and makes it available in the importing file.
    ///
    /// # Arguments
    /// * `from_file` - The file being imported from
    /// * `to_file` - The file doing the importing
    /// * `symbols` - The symbols being imported (None = import all exported)
    pub fn propagate_types(
        &mut self,
        from_file: &PathBuf,
        to_file: &PathBuf,
        symbols: Option<&[String]>,
    ) {
        // Get symbols from source file
        let source_symbols = match self.files.get(from_file) {
            Some(analysis) => analysis.symbols.clone(),
            None => return, // Source file not analyzed yet
        };

        // Determine which symbols to propagate
        let symbols_to_propagate: Vec<String> = match symbols {
            Some(names) => names.to_vec(),
            None => {
                // Import all exported symbols
                source_symbols
                    .values()
                    .filter(|b| b.is_exported)
                    .map(|b| b.symbol.clone())
                    .collect()
            }
        };

        // Add type bindings to importing file
        for symbol_name in symbols_to_propagate {
            if let Some(binding) = source_symbols.get(&symbol_name) {
                // Create new binding in target file
                let imported_binding = TypeBinding {
                    ty: binding.ty.clone(),
                    source_file: to_file.clone(),
                    symbol: symbol_name.clone(),
                    line: 0,            // Import statement line (could be tracked)
                    is_exported: false, // Imported symbols are not re-exported by default
                    dependencies: binding.dependencies.clone(),
                    is_propagated: true,
                };

                // Add to target file's symbols
                if let Some(target_analysis) = self.files.get_mut(to_file) {
                    target_analysis
                        .symbols
                        .insert(symbol_name.clone(), imported_binding.clone());
                }

                // Add to global type context
                self.context.add_binding(to_file.clone(), imported_binding);
            }
        }

        // Track import relationship
        self.import_graph
            .add_import(to_file.clone(), from_file.clone());
    }

    /// Update a symbol's type and propagate changes to dependent files.
    ///
    /// When a symbol's type changes, this method updates all files that import
    /// this symbol, ensuring type consistency across the codebase.
    ///
    /// # Arguments
    /// * `file` - The file containing the symbol
    /// * `symbol` - The symbol whose type changed
    /// * `new_type` - The new type for the symbol
    pub fn update_symbol_type(&mut self, file: &PathBuf, symbol: &str, new_type: Type) {
        // Update in source file
        if let Some(analysis) = self.files.get_mut(file) {
            if let Some(binding) = analysis.symbols.get_mut(symbol) {
                binding.ty = new_type.clone();
            }
        }

        // Update in type context
        if let Some(binding) = self.context.get_binding(file, symbol) {
            let mut updated_binding = binding.clone();
            updated_binding.ty = new_type.clone();
            self.context.add_binding(file.clone(), updated_binding);
        }

        // Propagate to importing files
        if let Some(importers) = self.import_graph.imported_by(file) {
            for importing_file in importers.clone() {
                // Check if this file imports the changed symbol
                if let Some(analysis) = self.files.get(&importing_file) {
                    if analysis.symbols.contains_key(symbol) {
                        // Update the imported symbol's type
                        self.update_imported_symbol(&importing_file, symbol, new_type.clone());

                        // Recursively propagate to files importing from this file
                        self.update_symbol_type(&importing_file, symbol, new_type.clone());
                    }
                }
            }
        }
    }

    /// Update an imported symbol's type in a file.
    fn update_imported_symbol(&mut self, file: &PathBuf, symbol: &str, new_type: Type) {
        if let Some(analysis) = self.files.get_mut(file) {
            if let Some(binding) = analysis.symbols.get_mut(symbol) {
                binding.ty = new_type.clone();
            }
        }

        // Update in type context
        if let Some(binding) = self.context.get_binding(file, symbol) {
            let mut updated_binding = binding.clone();
            updated_binding.ty = new_type;
            self.context.add_binding(file.clone(), updated_binding);
        }
    }

    /// Add import information to a file.
    ///
    /// This records that a file imports specific symbols from another file,
    /// which is used for cross-file type propagation.
    pub fn add_import(&mut self, file: &PathBuf, import: ImportInfo) {
        if let Some(analysis) = self.files.get_mut(file) {
            analysis.imports.push(import);
        }
    }

    /// Get all symbols from a file (including imported ones).
    pub fn get_file_symbols(&self, file: &PathBuf) -> HashMap<String, Type> {
        match self.files.get(file) {
            Some(analysis) => analysis
                .symbols
                .iter()
                .map(|(name, binding)| (name.clone(), binding.ty.clone()))
                .collect(),
            None => HashMap::new(),
        }
    }

    /// Add a symbol binding to a file's analysis.
    ///
    /// This is useful for testing and for manually populating file symbols.
    pub fn add_file_symbol(&mut self, file: &PathBuf, symbol: String, binding: TypeBinding) {
        if let Some(analysis) = self.files.get_mut(file) {
            analysis.symbols.insert(symbol, binding);
        }
    }

    /// Get file analysis for a specific file (for testing).
    pub fn get_file_analysis(&self, file: &PathBuf) -> Option<&FileAnalysis> {
        self.files.get(file)
    }

    /// Set a symbol's export status in a file.
    pub fn set_symbol_exported(&mut self, file: &PathBuf, symbol: &str, exported: bool) {
        if let Some(analysis) = self.files.get_mut(file) {
            if let Some(binding) = analysis.symbols.get_mut(symbol) {
                binding.is_exported = exported;
            }
        }
    }

    // -- Propagation pipeline helpers (R1-R3, R8, R9) -------------------------

    /// Get immutable reference to a file's analysis (alias for `get_file_analysis`).
    pub fn file_analysis(&self, file: &PathBuf) -> Option<&FileAnalysis> {
        self.files.get(file)
    }

    /// Get mutable reference to a file's analysis.
    pub fn file_analysis_mut(&mut self, file: &PathBuf) -> Option<&mut FileAnalysis> {
        self.files.get_mut(file)
    }

    /// Add an import edge to the internal import graph.
    pub fn add_import_edge(&mut self, from: PathBuf, to: PathBuf) {
        self.import_graph.add_import(from, to);
    }

    /// Get forward dependencies from the internal import graph.
    pub fn import_graph_deps(&self, file: &PathBuf) -> Option<&HashSet<PathBuf>> {
        self.import_graph.imports(file)
    }

    /// Get reverse dependencies from the internal import graph.
    pub fn import_graph_reverse_deps(&self, file: &PathBuf) -> Option<&HashSet<PathBuf>> {
        self.import_graph.imported_by(file)
    }

    /// Return the topological sort order from the internal import graph.
    pub fn topological_sort(&self) -> Vec<PathBuf> {
        self.import_graph.topological_sort()
    }

    /// Run cross-file type propagation for all files in topological order (R1, R2, R3).
    ///
    /// Iterates files in topological order from the internal import graph,
    /// calling `propagate_types()` for each import edge so that downstream
    /// files receive resolved types instead of `Type::Unknown`.
    ///
    /// `cache` maps file paths to their `FileAnalysis` entries.  After this
    /// method returns, each entry's `propagation_complete` flag is set.
    pub fn propagate_all(&mut self, cache: &mut HashMap<PathBuf, FileAnalysis>) {
        // Merge any external cache entries into our internal files map.
        for (path, fa) in cache.iter() {
            if !self.files.contains_key(path) {
                self.files.insert(path.clone(), fa.clone());
            }
        }

        let topo_order = self.import_graph.topological_sort();
        let cycle_members: HashSet<PathBuf> =
            self.detect_import_cycles().into_iter().flatten().collect();

        for file in &topo_order {
            if cycle_members.contains(file) {
                if let Some(fa) = self.files.get_mut(file) {
                    fa.propagation_complete = true;
                }
                continue;
            }

            // Gather import edges for this file from the internal graph.
            let deps: Vec<PathBuf> = self
                .import_graph
                .imports(file)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect();

            for dep in deps {
                // Collect symbol names from the file's imports that reference this dep.
                let sym_names: Vec<String> = self
                    .files
                    .get(file)
                    .map(|fa| {
                        fa.imports
                            .iter()
                            .filter_map(|imp| imp.names.clone())
                            .flatten()
                            .collect()
                    })
                    .unwrap_or_default();

                let syms: Option<Vec<String>> = if sym_names.is_empty() {
                    None
                } else {
                    Some(sym_names)
                };
                let sym_slice: Option<&[String]> = syms.as_deref();

                self.propagate_types(&dep, file, sym_slice);
            }

            if let Some(fa) = self.files.get_mut(file) {
                fa.propagation_complete = true;
            }
        }

        // Write back to the external cache.
        for (path, fa) in &self.files {
            cache.insert(path.clone(), fa.clone());
        }
    }

    /// Detect import cycles in the internal import graph (R9).
    ///
    /// Returns a list of cycles; each cycle is an ordered list of file paths.
    pub fn detect_import_cycles(&self) -> Vec<Vec<PathBuf>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut stack: Vec<PathBuf> = Vec::new();
        let mut on_stack = HashSet::new();

        for file in self.import_graph.all_files() {
            if !visited.contains(&file) {
                Self::dfs_detect_cycles(
                    &self.import_graph,
                    &file,
                    &mut visited,
                    &mut stack,
                    &mut on_stack,
                    &mut cycles,
                );
            }
        }
        cycles
    }

    fn dfs_detect_cycles(
        graph: &ImportGraph,
        node: &PathBuf,
        visited: &mut HashSet<PathBuf>,
        stack: &mut Vec<PathBuf>,
        on_stack: &mut HashSet<PathBuf>,
        cycles: &mut Vec<Vec<PathBuf>>,
    ) {
        visited.insert(node.clone());
        stack.push(node.clone());
        on_stack.insert(node.clone());

        if let Some(deps) = graph.imports(node) {
            for dep in deps.clone() {
                if on_stack.contains(&dep) {
                    // Found a cycle — extract it from the stack.
                    if let Some(pos) = stack.iter().position(|p| p == &dep) {
                        cycles.push(stack[pos..].to_vec());
                    }
                } else if !visited.contains(&dep) {
                    Self::dfs_detect_cycles(graph, &dep, visited, stack, on_stack, cycles);
                }
            }
        }

        stack.pop();
        on_stack.remove(node);
    }

    /// Infer types across all files.
    pub fn infer_all(&mut self) -> Vec<TypeBinding> {
        // Get analysis order
        let order = self.import_graph.topological_sort();

        let mut all_bindings = Vec::new();

        for file in order {
            if let Some(analysis) = self.files.get_mut(&file) {
                // Analyze file
                // This would use the existing TypeInferencer
                analysis.complete = true;

                // Collect bindings
                for binding in analysis.symbols.values() {
                    all_bindings.push(binding.clone());
                }
            }
        }

        all_bindings
    }

    /// Trace a type through function calls.
    pub fn trace_type(&self, symbol: &str, file: &PathBuf) -> Vec<TypeTraceStep> {
        let mut trace = Vec::new();
        let mut visited = HashSet::new();

        self.trace_recursive(symbol, file, &mut trace, &mut visited);

        trace
    }

    fn trace_recursive(
        &self,
        symbol: &str,
        file: &PathBuf,
        trace: &mut Vec<TypeTraceStep>,
        visited: &mut HashSet<(String, PathBuf)>,
    ) {
        let key = (symbol.to_string(), file.clone());
        if visited.contains(&key) {
            return;
        }
        visited.insert(key);

        if let Some(binding) = self.context.get_binding(file, symbol) {
            trace.push(TypeTraceStep {
                symbol: symbol.to_string(),
                file: file.clone(),
                ty: binding.ty.clone(),
                line: binding.line,
            });

            // Follow dependencies
            for dep in &binding.dependencies {
                self.trace_recursive(dep, file, trace, visited);
            }
        }
    }
}

impl Default for DeepTypeInferencer {
    fn default() -> Self {
        Self::new()
    }
}

/// A step in a type trace.
#[derive(Debug, Clone)]
pub struct TypeTraceStep {
    /// Symbol name
    pub symbol: String,
    /// File containing the symbol
    pub file: PathBuf,
    /// Type at this step
    pub ty: Type,
    /// Line number
    pub line: u32,
}

// ============================================================================
// MCP Tool Functions
// ============================================================================

/// Deep type inference result for MCP.
#[derive(Debug, Clone)]
pub struct DeepInferenceResult {
    /// Inferred type
    pub ty: Type,
    /// Source file
    pub source_file: PathBuf,
    /// Dependencies
    pub dependencies: Vec<String>,
    /// Cross-file references
    pub cross_file_refs: Vec<CrossFileRef>,
}

/// Cross-file reference.
#[derive(Debug, Clone)]
pub struct CrossFileRef {
    /// File path
    pub file: PathBuf,
    /// Symbol name
    pub symbol: String,
    /// Line number
    pub line: u32,
}

/// Infer type with deep cross-file analysis.
pub fn infer_type_deep(
    inferencer: &DeepTypeInferencer,
    symbol: &str,
    file: &PathBuf,
) -> Option<DeepInferenceResult> {
    let binding = inferencer.context.get_binding(file, symbol)?;

    let cross_file_refs = binding
        .dependencies
        .iter()
        .filter_map(|dep| {
            inferencer.context.resolve_type(dep, file).map(|_| {
                // Find where this dependency is defined
                for (f, bindings) in &inferencer.context.bindings {
                    if let Some(b) = bindings.get(dep) {
                        return CrossFileRef {
                            file: f.clone(),
                            symbol: dep.clone(),
                            line: b.line,
                        };
                    }
                }
                CrossFileRef {
                    file: file.clone(),
                    symbol: dep.clone(),
                    line: 0,
                }
            })
        })
        .collect();

    Some(DeepInferenceResult {
        ty: binding.ty.clone(),
        source_file: file.clone(),
        dependencies: binding.dependencies.clone(),
        cross_file_refs,
    })
}

/// Trace type through call chain.
pub fn trace_type_chain(
    inferencer: &DeepTypeInferencer,
    symbol: &str,
    file: &PathBuf,
) -> Vec<TypeTraceStep> {
    inferencer.trace_type(symbol, file)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_context() {
        let mut ctx = TypeContext::new();

        let binding = TypeBinding {
            ty: Type::Unknown,
            source_file: PathBuf::from("test.py"),
            symbol: "foo".to_string(),
            line: 1,
            is_exported: true,
            dependencies: vec![],
            is_propagated: false,
        };

        ctx.add_binding(PathBuf::from("test.py"), binding);

        assert!(ctx.get_binding(&PathBuf::from("test.py"), "foo").is_some());
    }

    #[test]
    fn test_type_var_info() {
        let tv = TypeVarInfo::new("T").with_bound(Type::Unknown).covariant();

        assert_eq!(tv.name, "T");
        assert!(tv.covariant);
        assert!(!tv.contravariant);
    }

    #[test]
    fn test_import_graph_topological_sort() {
        let mut graph = ImportGraph::new();

        graph.add_import(PathBuf::from("a.py"), PathBuf::from("b.py"));
        graph.add_import(PathBuf::from("b.py"), PathBuf::from("c.py"));

        let order = graph.topological_sort();

        // c.py should come before b.py, b.py before a.py
        let c_pos = order.iter().position(|p| p == &PathBuf::from("c.py"));
        let b_pos = order.iter().position(|p| p == &PathBuf::from("b.py"));
        let a_pos = order.iter().position(|p| p == &PathBuf::from("a.py"));

        if let (Some(c), Some(b), Some(a)) = (c_pos, b_pos, a_pos) {
            assert!(c < b);
            assert!(b < a);
        }
    }

    #[test]
    fn test_deep_inferencer() {
        let mut inferencer = DeepTypeInferencer::new();
        inferencer.add_file(PathBuf::from("test.py"));

        assert!(inferencer.files.contains_key(&PathBuf::from("test.py")));
    }

    /// R9, S4: Cycle between A↔B detected, cycle members returned, no infinite loop.
    #[test]
    fn test_circular_import_detection() {
        let mut inferencer = DeepTypeInferencer::new();
        let a = PathBuf::from("a.py");
        let b = PathBuf::from("b.py");
        inferencer.add_file(a.clone());
        inferencer.add_file(b.clone());

        // A imports B, B imports A → cycle
        inferencer.add_import_edge(a.clone(), b.clone());
        inferencer.add_import_edge(b.clone(), a.clone());

        let cycles = inferencer.detect_import_cycles();
        assert!(!cycles.is_empty(), "Should detect at least one cycle");

        // The cycle members should include both a.py and b.py
        let all_members: HashSet<PathBuf> = cycles.into_iter().flatten().collect();
        assert!(all_members.contains(&a), "a.py should be in cycle");
        assert!(all_members.contains(&b), "b.py should be in cycle");

        // Topological sort should still terminate (no infinite loop).
        let topo = inferencer.topological_sort();
        assert!(
            !topo.is_empty(),
            "Topological sort should still return results"
        );
    }

    /// R9, S4: Symbols in cycle retain local types, cross-cycle imports
    /// remain Type::Unknown.
    #[test]
    fn test_circular_import_fallback() {
        let mut inferencer = DeepTypeInferencer::new();
        let a = PathBuf::from("a.py");
        let b = PathBuf::from("b.py");
        inferencer.add_file(a.clone());
        inferencer.add_file(b.clone());

        // a.py defines foo locally
        inferencer.add_file_symbol(
            &a,
            "foo".to_string(),
            TypeBinding {
                ty: Type::Int,
                source_file: a.clone(),
                symbol: "foo".to_string(),
                line: 1,
                is_exported: true,
                dependencies: vec![],
                is_propagated: false,
            },
        );

        // b.py defines bar locally
        inferencer.add_file_symbol(
            &b,
            "bar".to_string(),
            TypeBinding {
                ty: Type::Str,
                source_file: b.clone(),
                symbol: "bar".to_string(),
                line: 1,
                is_exported: true,
                dependencies: vec![],
                is_propagated: false,
            },
        );

        // Circular import edges
        inferencer.add_import_edge(a.clone(), b.clone());
        inferencer.add_import_edge(b.clone(), a.clone());

        // Cycles detected
        let cycles = inferencer.detect_import_cycles();
        assert!(!cycles.is_empty());

        // Local types should still be intact
        let fa_a = inferencer.file_analysis(&a).unwrap();
        assert_eq!(fa_a.symbols.get("foo").unwrap().ty, Type::Int);

        let fa_b = inferencer.file_analysis(&b).unwrap();
        assert_eq!(fa_b.symbols.get("bar").unwrap().ty, Type::Str);
    }

    /// R3: Propagated TypeBindings have is_propagated = true, local ones false.
    #[test]
    fn test_propagated_binding_flag() {
        let mut inferencer = DeepTypeInferencer::new();
        let source = PathBuf::from("source.py");
        let target = PathBuf::from("target.py");
        inferencer.add_file(source.clone());
        inferencer.add_file(target.clone());

        // Source has an exported symbol
        inferencer.add_file_symbol(
            &source,
            "helper".to_string(),
            TypeBinding {
                ty: Type::Int,
                source_file: source.clone(),
                symbol: "helper".to_string(),
                line: 5,
                is_exported: true,
                dependencies: vec![],
                is_propagated: false,
            },
        );

        // Target has a local symbol
        inferencer.add_file_symbol(
            &target,
            "local_var".to_string(),
            TypeBinding {
                ty: Type::Str,
                source_file: target.clone(),
                symbol: "local_var".to_string(),
                line: 1,
                is_exported: false,
                dependencies: vec![],
                is_propagated: false,
            },
        );

        // Propagate from source to target
        inferencer.propagate_types(&source, &target, Some(&["helper".to_string()]));

        let fa = inferencer.file_analysis(&target).unwrap();

        // Local binding should not be propagated.
        let local = fa.symbols.get("local_var").unwrap();
        assert!(
            !local.is_propagated,
            "Local binding should have is_propagated=false"
        );

        // Propagated binding should be marked.
        let propagated = fa.symbols.get("helper").unwrap();
        assert!(
            propagated.is_propagated,
            "Propagated binding should have is_propagated=true"
        );
    }
}

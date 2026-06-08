//! Rust symbol collection from AST
//!
//! This module provides functionality for extracting Rust symbols
//! (structs, enums, traits, impl blocks, functions, etc.) from tree-sitter AST.

use tree_sitter::Node;

use super::rust_types::*;
use super::ty::TypeVarId;

// ============================================================================
// Symbol Types
// ============================================================================

/// Collected symbols from a Rust file
#[derive(Debug, Clone, Default)]
pub struct RustSymbols {
    /// Struct definitions
    pub structs: Vec<StructDef>,
    /// Enum definitions
    pub enums: Vec<EnumDef>,
    /// Trait definitions
    pub traits: Vec<TraitDef>,
    /// Impl blocks
    pub impls: Vec<ImplBlock>,
    /// Function definitions
    pub functions: Vec<RustFunction>,
    /// Constants
    pub constants: Vec<RustConstant>,
    /// Type aliases
    pub type_aliases: Vec<RustTypeAlias>,
}

/// Rust function definition
#[derive(Debug, Clone)]
pub struct RustFunction {
    /// Function name
    pub name: String,
    /// Visibility
    pub visibility: Visibility,
    /// Generic parameters
    pub type_params: Vec<RustTypeParam>,
    /// Parameters
    pub params: Vec<RustParam>,
    /// Return type
    pub return_type: RustType,
    /// Where clause
    pub where_bounds: Vec<WherePredicate>,
    /// Is async
    pub is_async: bool,
    /// Is unsafe
    pub is_unsafe: bool,
    /// Is const
    pub is_const: bool,
    /// Source span (start, end)
    pub span: (usize, usize),
}

/// Rust constant definition
#[derive(Debug, Clone)]
pub struct RustConstant {
    /// Constant name
    pub name: String,
    /// Visibility
    pub visibility: Visibility,
    /// Type
    pub ty: RustType,
    /// Source span
    pub span: (usize, usize),
}

/// Rust type alias
#[derive(Debug, Clone)]
pub struct RustTypeAlias {
    /// Alias name
    pub name: String,
    /// Visibility
    pub visibility: Visibility,
    /// Generic parameters
    pub type_params: Vec<RustTypeParam>,
    /// Target type
    pub target: RustType,
    /// Source span
    pub span: (usize, usize),
}

// ============================================================================
// Symbol Collector
// ============================================================================

/// Collects Rust symbols from AST
pub struct RustSymbolCollector {
    /// Collected symbols
    symbols: RustSymbols,
    /// Trait ID counter
    trait_id_counter: usize,
    /// Type var counter
    type_var_counter: usize,
}

impl RustSymbolCollector {
    /// Create a new symbol collector
    pub fn new() -> Self {
        Self {
            symbols: RustSymbols::default(),
            trait_id_counter: 0,
            type_var_counter: 0,
        }
    }

    /// Collect symbols from a tree-sitter node (source_file)
    pub fn collect(&mut self, node: &Node, source: &str) -> RustSymbols {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            self.collect_item(&child, source);
        }

        std::mem::take(&mut self.symbols)
    }

    /// Collect a single item
    fn collect_item(&mut self, node: &Node, source: &str) {
        match node.kind() {
            "struct_item" => self.collect_struct(node, source),
            "enum_item" => self.collect_enum(node, source),
            "trait_item" => self.collect_trait(node, source),
            "impl_item" => self.collect_impl(node, source),
            "function_item" => self.collect_function(node, source),
            "const_item" => self.collect_const(node, source),
            "type_item" => self.collect_type_alias(node, source),
            "mod_item" => self.collect_mod(node, source),
            _ => {}
        }
    }

    /// Collect struct definition
    fn collect_struct(&mut self, node: &Node, source: &str) {
        let name = self.get_name(node, source);
        let visibility = self.get_visibility(node);
        let type_params = self.collect_type_params(node, source);
        let fields = self.collect_struct_fields(node, source);
        let where_bounds = self.collect_where_clause(node, source);

        self.symbols.structs.push(StructDef {
            name,
            module: None,
            type_params,
            fields,
            where_bounds,
            visibility,
        });
    }

    /// Collect enum definition
    fn collect_enum(&mut self, node: &Node, source: &str) {
        let name = self.get_name(node, source);
        let visibility = self.get_visibility(node);
        let type_params = self.collect_type_params(node, source);
        let variants = self.collect_enum_variants(node, source);
        let where_bounds = self.collect_where_clause(node, source);

        self.symbols.enums.push(EnumDef {
            name,
            module: None,
            type_params,
            variants,
            where_bounds,
            visibility,
        });
    }

    /// Collect trait definition
    fn collect_trait(&mut self, node: &Node, source: &str) {
        let name = self.get_name(node, source);
        let id = TraitId(self.trait_id_counter);
        self.trait_id_counter += 1;

        let type_params = self.collect_type_params(node, source);
        let supertraits = self.collect_supertraits(node, source);

        self.symbols.traits.push(TraitDef {
            id,
            name,
            module: None,
            type_params,
            supertraits,
            associated_types: vec![],
            required_methods: vec![],
            provided_methods: vec![],
            is_auto: false,
            is_marker: false,
        });
    }

    /// Collect impl block
    fn collect_impl(&mut self, node: &Node, source: &str) {
        let type_params = self.collect_type_params(node, source);
        let trait_ref = self.collect_impl_trait(node, source);
        let self_type = self.collect_impl_type(node, source);
        let where_bounds = self.collect_where_clause(node, source);
        let methods = self.collect_impl_methods(node, source);

        self.symbols.impls.push(ImplBlock {
            type_params,
            trait_ref,
            self_type,
            where_bounds,
            methods,
            associated_types: vec![],
            associated_consts: vec![],
            is_negative: false,
            is_unsafe: node.child_by_field_name("unsafe").is_some(),
        });
    }

    /// Collect function definition
    fn collect_function(&mut self, node: &Node, source: &str) {
        let name = self.get_name(node, source);
        let visibility = self.get_visibility(node);
        let type_params = self.collect_type_params(node, source);
        let params = self.collect_function_params(node, source);
        let return_type = self.collect_return_type(node, source);
        let where_bounds = self.collect_where_clause(node, source);

        self.symbols.functions.push(RustFunction {
            name,
            visibility,
            type_params,
            params,
            return_type,
            where_bounds,
            is_async: node.child_by_field_name("async").is_some(),
            is_unsafe: node.child_by_field_name("unsafe").is_some(),
            is_const: node.child_by_field_name("const").is_some(),
            span: (node.start_byte(), node.end_byte()),
        });
    }

    /// Collect const definition
    fn collect_const(&mut self, node: &Node, source: &str) {
        let name = self.get_name(node, source);
        let visibility = self.get_visibility(node);
        let ty = if let Some(type_node) = node.child_by_field_name("type") {
            self.parse_type(&type_node, source)
        } else {
            RustType::Infer
        };

        self.symbols.constants.push(RustConstant {
            name,
            visibility,
            ty,
            span: (node.start_byte(), node.end_byte()),
        });
    }

    /// Collect type alias
    fn collect_type_alias(&mut self, node: &Node, source: &str) {
        let name = self.get_name(node, source);
        let visibility = self.get_visibility(node);
        let type_params = self.collect_type_params(node, source);
        let target = if let Some(type_node) = node.child_by_field_name("type") {
            self.parse_type(&type_node, source)
        } else {
            RustType::Infer
        };

        self.symbols.type_aliases.push(RustTypeAlias {
            name,
            visibility,
            type_params,
            target,
            span: (node.start_byte(), node.end_byte()),
        });
    }

    /// Collect items from a module
    fn collect_mod(&mut self, node: &Node, source: &str) {
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                self.collect_item(&child, source);
            }
        }
    }

    // ========================================================================
    // Helper methods
    // ========================================================================

    fn get_name(&self, node: &Node, source: &str) -> String {
        node.child_by_field_name("name")
            .map(|n| source[n.start_byte()..n.end_byte()].to_string())
            .unwrap_or_default()
    }

    fn get_visibility(&self, node: &Node) -> Visibility {
        if let Some(vis) = node.child_by_field_name("visibility_modifier") {
            let mut cursor = vis.walk();
            for child in vis.children(&mut cursor) {
                match child.kind() {
                    "crate" => return Visibility::Crate,
                    "super" => return Visibility::Super,
                    _ => {}
                }
            }
            Visibility::Public
        } else {
            Visibility::Private
        }
    }

    fn collect_type_params(&mut self, node: &Node, source: &str) -> Vec<RustTypeParam> {
        let mut params = Vec::new();
        if let Some(params_node) = node.child_by_field_name("type_parameters") {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                match child.kind() {
                    "type_identifier" => {
                        let name = source[child.start_byte()..child.end_byte()].to_string();
                        let id = TypeVarId(self.type_var_counter);
                        self.type_var_counter += 1;
                        params.push(RustTypeParam {
                            name,
                            id,
                            bounds: vec![],
                            default: None,
                        });
                    }
                    "constrained_type_parameter" => {
                        // Handle T: Bound syntax
                        if let Some(name_node) = child.child_by_field_name("left") {
                            let name =
                                source[name_node.start_byte()..name_node.end_byte()].to_string();
                            let id = TypeVarId(self.type_var_counter);
                            self.type_var_counter += 1;

                            let mut bounds = Vec::new();
                            if let Some(bounds_node) = child.child_by_field_name("bounds") {
                                bounds = self.parse_trait_bounds(&bounds_node, source);
                            }

                            params.push(RustTypeParam {
                                name,
                                id,
                                bounds,
                                default: None,
                            });
                        }
                    }
                    "optional_type_parameter" => {
                        // Handle T = Default syntax
                        if let Some(name_node) = child.child_by_field_name("name") {
                            let name =
                                source[name_node.start_byte()..name_node.end_byte()].to_string();
                            let id = TypeVarId(self.type_var_counter);
                            self.type_var_counter += 1;

                            let default = child
                                .child_by_field_name("default_type")
                                .map(|n| self.parse_type(&n, source));

                            params.push(RustTypeParam {
                                name,
                                id,
                                bounds: vec![],
                                default,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
        params
    }

    fn collect_struct_fields(&self, node: &Node, source: &str) -> StructFields {
        if let Some(body) = node.child_by_field_name("body") {
            match body.kind() {
                "field_declaration_list" => {
                    let mut fields = Vec::new();
                    let mut cursor = body.walk();
                    for child in body.children(&mut cursor) {
                        if child.kind() == "field_declaration" {
                            if let Some(field) = self.parse_field(&child, source) {
                                fields.push(field);
                            }
                        }
                    }
                    StructFields::Named(fields)
                }
                "ordered_field_declaration_list" => {
                    let mut types = Vec::new();
                    let mut cursor = body.walk();
                    for child in body.children(&mut cursor) {
                        if !matches!(child.kind(), "(" | ")" | ",") {
                            types.push(self.parse_type(&child, source));
                        }
                    }
                    StructFields::Tuple(types)
                }
                _ => StructFields::Unit,
            }
        } else {
            StructFields::Unit
        }
    }

    fn parse_field(&self, node: &Node, source: &str) -> Option<StructField> {
        let name = node
            .child_by_field_name("name")
            .map(|n| source[n.start_byte()..n.end_byte()].to_string())?;
        let ty = node
            .child_by_field_name("type")
            .map(|n| self.parse_type(&n, source))
            .unwrap_or(RustType::Infer);
        let visibility = self.get_visibility(node);

        Some(StructField {
            name,
            ty,
            visibility,
        })
    }

    fn collect_enum_variants(&self, node: &Node, source: &str) -> Vec<EnumVariant> {
        let mut variants = Vec::new();
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "enum_variant" {
                    let name = child
                        .child_by_field_name("name")
                        .map(|n| source[n.start_byte()..n.end_byte()].to_string())
                        .unwrap_or_default();
                    let fields = self.collect_variant_fields(&child, source);
                    let discriminant = self.parse_discriminant(&child, source);
                    variants.push(EnumVariant {
                        name,
                        fields,
                        discriminant,
                    });
                }
            }
        }
        variants
    }

    fn parse_discriminant(&self, node: &Node, source: &str) -> Option<i128> {
        // Look for the value field in enum_variant which contains the discriminant
        node.child_by_field_name("value").and_then(|value_node| {
            let text = source[value_node.start_byte()..value_node.end_byte()].trim();

            // Try to parse as integer (handles decimal, hex, octal, binary)
            if text.starts_with("0x") || text.starts_with("0X") {
                i128::from_str_radix(&text[2..].replace('_', ""), 16).ok()
            } else if text.starts_with("0o") || text.starts_with("0O") {
                i128::from_str_radix(&text[2..].replace('_', ""), 8).ok()
            } else if text.starts_with("0b") || text.starts_with("0B") {
                i128::from_str_radix(&text[2..].replace('_', ""), 2).ok()
            } else {
                text.replace('_', "").parse::<i128>().ok()
            }
        })
    }

    fn collect_variant_fields(&self, node: &Node, source: &str) -> StructFields {
        if let Some(body) = node.child_by_field_name("body") {
            self.collect_struct_fields_from_body(&body, source)
        } else {
            StructFields::Unit
        }
    }

    fn collect_struct_fields_from_body(&self, body: &Node, source: &str) -> StructFields {
        match body.kind() {
            "field_declaration_list" => {
                let mut fields = Vec::new();
                let mut cursor = body.walk();
                for child in body.children(&mut cursor) {
                    if child.kind() == "field_declaration" {
                        if let Some(field) = self.parse_field(&child, source) {
                            fields.push(field);
                        }
                    }
                }
                StructFields::Named(fields)
            }
            "ordered_field_declaration_list" => {
                let mut types = Vec::new();
                let mut cursor = body.walk();
                for child in body.children(&mut cursor) {
                    if !matches!(child.kind(), "(" | ")" | ",") {
                        types.push(self.parse_type(&child, source));
                    }
                }
                StructFields::Tuple(types)
            }
            _ => StructFields::Unit,
        }
    }

    fn collect_supertraits(&self, node: &Node, source: &str) -> Vec<TraitBound> {
        let mut bounds = Vec::new();

        // Look for trait_bounds node (after the colon in trait definition)
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "trait_bounds" {
                bounds.extend(self.parse_trait_bounds(&child, source));
            }
        }

        bounds
    }

    fn parse_trait_bounds(&self, node: &Node, source: &str) -> Vec<TraitBound> {
        let mut bounds = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "type_identifier" | "scoped_type_identifier" | "generic_type" => {
                    let trait_ref = self.parse_trait_ref(&child, source);
                    bounds.push(TraitBound {
                        trait_ref,
                        is_negative: false,
                        higher_ranked_lifetimes: vec![],
                    });
                }
                "removed_trait_bound" => {
                    // Negative bound like !Send
                    if let Some(inner) = child.child(1) {
                        let trait_ref = self.parse_trait_ref(&inner, source);
                        bounds.push(TraitBound {
                            trait_ref,
                            is_negative: true,
                            higher_ranked_lifetimes: vec![],
                        });
                    }
                }
                "higher_ranked_trait_bound" => {
                    // for<'a> Trait<'a>
                    let lifetimes = self.collect_higher_ranked_lifetimes(&child, source);
                    if let Some(trait_node) = child.child_by_field_name("type") {
                        let trait_ref = self.parse_trait_ref(&trait_node, source);
                        bounds.push(TraitBound {
                            trait_ref,
                            is_negative: false,
                            higher_ranked_lifetimes: lifetimes,
                        });
                    }
                }
                _ => {}
            }
        }

        bounds
    }

    fn parse_trait_ref(&self, node: &Node, source: &str) -> TraitRef {
        let name = source[node.start_byte()..node.end_byte()].to_string();
        let mut type_args = Vec::new();
        let mut lifetime_args = Vec::new();

        // Check for generic arguments
        if node.kind() == "generic_type" {
            if let Some(args_node) = node.child_by_field_name("type_arguments") {
                let mut cursor = args_node.walk();
                for child in args_node.children(&mut cursor) {
                    match child.kind() {
                        "lifetime" => {
                            let lt_name = source[child.start_byte()..child.end_byte()].to_string();
                            lifetime_args.push(Lifetime::Named {
                                id: LifetimeId(0),
                                name: lt_name,
                            });
                        }
                        _ if !matches!(child.kind(), "<" | ">" | ",") => {
                            type_args.push(self.parse_type(&child, source));
                        }
                        _ => {}
                    }
                }
            }
        }

        TraitRef {
            trait_id: TraitId(0), // Will be resolved during semantic analysis
            name,
            type_args,
            lifetime_args,
        }
    }

    fn collect_higher_ranked_lifetimes(&self, node: &Node, source: &str) -> Vec<Lifetime> {
        let mut lifetimes = Vec::new();

        if let Some(params) = node.child_by_field_name("type_parameters") {
            let mut cursor = params.walk();
            for child in params.children(&mut cursor) {
                if child.kind() == "lifetime" {
                    let name = source[child.start_byte()..child.end_byte()].to_string();
                    lifetimes.push(Lifetime::Named {
                        id: LifetimeId(0),
                        name,
                    });
                }
            }
        }

        lifetimes
    }

    fn collect_impl_trait(&self, node: &Node, source: &str) -> Option<TraitRef> {
        node.child_by_field_name("trait")
            .map(|n| self.parse_trait_ref(&n, source))
    }

    fn collect_impl_type(&self, node: &Node, source: &str) -> RustType {
        node.child_by_field_name("type")
            .map(|n| self.parse_type(&n, source))
            .unwrap_or(RustType::Infer)
    }

    fn collect_where_clause(&self, node: &Node, source: &str) -> Vec<WherePredicate> {
        let mut predicates = Vec::new();

        // Find the where_clause child
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "where_clause" {
                predicates.extend(self.parse_where_clause(&child, source));
            }
        }

        predicates
    }

    fn parse_where_clause(&self, node: &Node, source: &str) -> Vec<WherePredicate> {
        let mut predicates = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "where_predicate" {
                if let Some(pred) = self.parse_where_predicate(&child, source) {
                    predicates.push(pred);
                }
            }
        }

        predicates
    }

    fn parse_where_predicate(&self, node: &Node, source: &str) -> Option<WherePredicate> {
        // Check for lifetime bound: 'a: 'b
        if let Some(lifetime) = node.child_by_field_name("left") {
            if lifetime.kind() == "lifetime" {
                let lt_name = source[lifetime.start_byte()..lifetime.end_byte()].to_string();
                let lifetime = Lifetime::Named {
                    id: LifetimeId(0),
                    name: lt_name,
                };

                let mut bounds = Vec::new();
                if let Some(bounds_node) = node.child_by_field_name("bounds") {
                    let mut cursor = bounds_node.walk();
                    for child in bounds_node.children(&mut cursor) {
                        if child.kind() == "lifetime" {
                            let name = source[child.start_byte()..child.end_byte()].to_string();
                            bounds.push(Lifetime::Named {
                                id: LifetimeId(0),
                                name,
                            });
                        }
                    }
                }

                return Some(WherePredicate::LifetimeBound { lifetime, bounds });
            }
        }

        // Type bound: T: Trait
        if let Some(type_node) = node.child_by_field_name("left") {
            let ty = self.parse_type(&type_node, source);

            let mut bounds = Vec::new();
            if let Some(bounds_node) = node.child_by_field_name("bounds") {
                bounds = self.parse_trait_bounds(&bounds_node, source);
            }

            return Some(WherePredicate::TypeBound { ty, bounds });
        }

        None
    }

    fn collect_impl_methods(&self, node: &Node, source: &str) -> Vec<ImplMethod> {
        let mut methods = Vec::new();
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "function_item" {
                    let name = child
                        .child_by_field_name("name")
                        .map(|n| source[n.start_byte()..n.end_byte()].to_string())
                        .unwrap_or_default();
                    let visibility = self.get_visibility(&child);
                    let type_params = self.collect_method_type_params(&child, source);
                    let params = self.collect_function_params(&child, source);
                    let where_bounds = self.collect_where_clause(&child, source);

                    methods.push(ImplMethod {
                        name: name.clone(),
                        visibility,
                        signature: TraitMethod {
                            name,
                            type_params,
                            self_param: self.get_self_param(&child),
                            params,
                            return_type: self.collect_return_type(&child, source),
                            where_bounds,
                            is_unsafe: child.child_by_field_name("unsafe").is_some(),
                            is_async: child.child_by_field_name("async").is_some(),
                        },
                        is_default: false,
                    });
                }
            }
        }
        methods
    }

    fn collect_method_type_params(&self, node: &Node, source: &str) -> Vec<RustTypeParam> {
        let mut params = Vec::new();
        if let Some(params_node) = node.child_by_field_name("type_parameters") {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                match child.kind() {
                    "type_identifier" => {
                        let name = source[child.start_byte()..child.end_byte()].to_string();
                        let id = TypeVarId(params.len());
                        params.push(RustTypeParam {
                            name,
                            id,
                            bounds: vec![],
                            default: None,
                        });
                    }
                    "constrained_type_parameter" => {
                        // T: Bound
                        if let Some(name_node) = child.child_by_field_name("left") {
                            let name =
                                source[name_node.start_byte()..name_node.end_byte()].to_string();
                            let id = TypeVarId(params.len());

                            let mut bounds = Vec::new();
                            if let Some(bounds_node) = child.child_by_field_name("bounds") {
                                bounds = self.parse_trait_bounds(&bounds_node, source);
                            }

                            params.push(RustTypeParam {
                                name,
                                id,
                                bounds,
                                default: None,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
        params
    }

    fn get_self_param(&self, node: &Node) -> Option<SelfParam> {
        if let Some(params) = node.child_by_field_name("parameters") {
            let mut cursor = params.walk();
            for child in params.children(&mut cursor) {
                if child.kind() == "self_parameter" {
                    // Check for & or &mut
                    let mut inner_cursor = child.walk();
                    for inner in child.children(&mut inner_cursor) {
                        if inner.kind() == "&" {
                            if child.child_by_field_name("mutable_specifier").is_some() {
                                return Some(SelfParam::RefMut(None));
                            } else {
                                return Some(SelfParam::Ref(None));
                            }
                        }
                    }
                    return Some(SelfParam::Value);
                }
            }
        }
        None
    }

    fn collect_function_params(&self, node: &Node, source: &str) -> Vec<RustParam> {
        let mut params = Vec::new();
        if let Some(params_node) = node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                if child.kind() == "parameter" {
                    let name = child
                        .child_by_field_name("pattern")
                        .map(|n| source[n.start_byte()..n.end_byte()].to_string())
                        .unwrap_or_default();
                    let ty = child
                        .child_by_field_name("type")
                        .map(|n| self.parse_type(&n, source))
                        .unwrap_or(RustType::Infer);
                    params.push(RustParam {
                        name,
                        ty,
                        is_pattern: false,
                    });
                }
            }
        }
        params
    }

    fn collect_return_type(&self, node: &Node, source: &str) -> RustType {
        node.child_by_field_name("return_type")
            .map(|n| self.parse_type(&n, source))
            .unwrap_or(RustType::Unit)
    }

    fn parse_type(&self, node: &Node, source: &str) -> RustType {
        let kind = node.kind();
        match kind {
            "primitive_type" => {
                let text = &source[node.start_byte()..node.end_byte()];
                match text {
                    "bool" => RustType::Bool,
                    "char" => RustType::Char,
                    "str" => RustType::Str,
                    "i8" => RustType::I8,
                    "i16" => RustType::I16,
                    "i32" => RustType::I32,
                    "i64" => RustType::I64,
                    "i128" => RustType::I128,
                    "isize" => RustType::Isize,
                    "u8" => RustType::U8,
                    "u16" => RustType::U16,
                    "u32" => RustType::U32,
                    "u64" => RustType::U64,
                    "u128" => RustType::U128,
                    "usize" => RustType::Usize,
                    "f32" => RustType::F32,
                    "f64" => RustType::F64,
                    _ => RustType::Infer,
                }
            }
            "type_identifier" | "scoped_type_identifier" => {
                let name = source[node.start_byte()..node.end_byte()].to_string();
                RustType::Named {
                    name,
                    module: None,
                    type_args: vec![],
                    lifetime_args: vec![],
                }
            }
            "reference_type" => {
                let mutable = node.child_by_field_name("mutable_specifier").is_some();
                let inner = node
                    .child_by_field_name("type")
                    .map(|n| self.parse_type(&n, source))
                    .unwrap_or(RustType::Infer);
                RustType::Reference {
                    lifetime: None,
                    mutable,
                    inner: Box::new(inner),
                }
            }
            "tuple_type" => {
                let mut elements = Vec::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if !matches!(child.kind(), "(" | ")" | ",") {
                        elements.push(self.parse_type(&child, source));
                    }
                }
                RustType::Tuple(elements)
            }
            "array_type" => {
                let element = node
                    .child_by_field_name("element")
                    .map(|n| self.parse_type(&n, source))
                    .unwrap_or(RustType::Infer);
                RustType::Array {
                    element: Box::new(element),
                    size: 0,
                }
            }
            "unit_type" => RustType::Unit,
            "never_type" => RustType::Never,
            _ => RustType::Infer,
        }
    }
}

impl Default for RustSymbolCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_symbols_default() {
        let symbols = RustSymbols::default();
        assert!(symbols.structs.is_empty());
        assert!(symbols.enums.is_empty());
        assert!(symbols.traits.is_empty());
        assert!(symbols.impls.is_empty());
        assert!(symbols.functions.is_empty());
    }

    #[test]
    fn test_symbol_collector_creation() {
        let collector = RustSymbolCollector::new();
        assert_eq!(collector.trait_id_counter, 0);
        assert_eq!(collector.type_var_counter, 0);
    }

    #[test]
    fn test_discriminant_parsing_logic() {
        // Test parsing discriminant values directly
        fn parse_value(text: &str) -> Option<i128> {
            let text = text.trim();
            if text.starts_with("0x") || text.starts_with("0X") {
                i128::from_str_radix(&text[2..].replace('_', ""), 16).ok()
            } else if text.starts_with("0o") || text.starts_with("0O") {
                i128::from_str_radix(&text[2..].replace('_', ""), 8).ok()
            } else if text.starts_with("0b") || text.starts_with("0B") {
                i128::from_str_radix(&text[2..].replace('_', ""), 2).ok()
            } else {
                text.replace('_', "").parse::<i128>().ok()
            }
        }

        // Test decimal
        assert_eq!(parse_value("42"), Some(42));

        // Test hex
        assert_eq!(parse_value("0xFF"), Some(255));

        // Test with underscores
        assert_eq!(parse_value("1_000_000"), Some(1_000_000));

        // Test binary
        assert_eq!(parse_value("0b1010"), Some(10));

        // Test octal
        assert_eq!(parse_value("0o77"), Some(63));

        // Test negative
        assert_eq!(parse_value("-1"), Some(-1));
    }

    #[test]
    fn test_trait_ref_creation() {
        // Test that TraitRef has proper defaults
        let trait_ref = TraitRef {
            trait_id: TraitId(1),
            name: "Clone".to_string(),
            type_args: vec![],
            lifetime_args: vec![],
        };
        assert_eq!(trait_ref.name, "Clone");
        assert!(trait_ref.type_args.is_empty());
    }

    #[test]
    fn test_where_predicate_types() {
        // Test TypeBound predicate
        let type_bound = WherePredicate::TypeBound {
            ty: RustType::Named {
                name: "T".to_string(),
                module: None,
                type_args: vec![],
                lifetime_args: vec![],
            },
            bounds: vec![TraitBound {
                trait_ref: TraitRef {
                    trait_id: TraitId(0),
                    name: "Clone".to_string(),
                    type_args: vec![],
                    lifetime_args: vec![],
                },
                is_negative: false,
                higher_ranked_lifetimes: vec![],
            }],
        };

        if let WherePredicate::TypeBound { ty, bounds } = type_bound {
            assert_eq!(bounds.len(), 1);
            assert!(!bounds[0].is_negative);
            if let RustType::Named { name, .. } = ty {
                assert_eq!(name, "T");
            }
        }

        // Test LifetimeBound predicate
        let lifetime_bound = WherePredicate::LifetimeBound {
            lifetime: Lifetime::Named {
                id: LifetimeId(0),
                name: "'a".to_string(),
            },
            bounds: vec![Lifetime::Static],
        };

        if let WherePredicate::LifetimeBound { lifetime, bounds } = lifetime_bound {
            assert_eq!(bounds.len(), 1);
            if let Lifetime::Named { name, .. } = lifetime {
                assert_eq!(name, "'a");
            }
        }
    }
}

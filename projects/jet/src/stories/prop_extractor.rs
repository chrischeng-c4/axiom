// <HANDWRITE gap="missing-generator:logic:962dae38" tracker="standardize-gap-projects-jet-src-stories-prop-extractor-rs" reason="Tree-sitter walk of a component file: locate the component's props type (interface or type alias referenced by the component's first param) and return an ordered list of (prop name, type text, optional flag).">
//! Component prop-type extraction for `jet stories` controls (B3).
//!
//! Given the source of a component file and the component's name, this module
//! finds the component's props type and returns one [`PropDef`] per declared
//! prop, preserving source order. The controls layer ([`super::controls`]) maps
//! each [`PropDef`] to an editable control widget.
//!
//! ## How the props type is found
//!
//! We locate the component declaration by name and read its first parameter's
//! type annotation, which names the props type. The common React shapes are:
//!
//! - `function Button(props: ButtonProps) { ... }`
//! - `const Button = (props: ButtonProps) => ...`
//! - `const Button: React.FC<ButtonProps> = (props) => ...`
//!   (the props type is the *type argument* of `React.FC<...>` / `FC<...>`)
//! - `function Button(props: { primary: boolean }) { ... }`
//!   (an inline object type — read directly)
//!
//! The named props type (`ButtonProps`) is then resolved against an `interface`
//! declaration or a `type` alias. The declaration is looked up **in the same
//! file** first, and — when the component's file path is known — against a
//! sibling file the props type is imported from.
//!
//! ## Cross-file / intersection / generic resolution (#198)
//!
//! [`extract_props_at`] threads the component's on-disk file path, which unlocks
//! three further shapes beyond the same-file simple case:
//!
//! - **Intersection** `type Props = Base & Extra`: each operand is resolved
//!   (same-file interface/type alias or an imported sibling type) and the
//!   members are unioned, de-duplicated by prop name (**first operand wins** on a
//!   name clash, mirroring TS's left-to-right declaration order for controls).
//! - **Cross-file imported prop type**: when the props type name is brought in by
//!   `import type { Props } from "./types"` / `import { Props } from "./types"`,
//!   the sibling file is resolved relative to the component file (project-local
//!   `.ts`/`.tsx`/`.d.ts`/… only; `node_modules` / bare specifiers are skipped),
//!   parsed, and that file's matching interface / type alias is read.
//! - **Generic** `Props<Variant>`: the generic interface/type alias is read with
//!   a best-effort substitution of the concrete type argument for the type
//!   parameter inside each member's type text. Members that don't reference the
//!   parameter are read verbatim.
//!
//! Every unresolved shape degrades to an empty/partial result rather than
//! erroring, so controls fall back to "no props" instead of failing the preview.
//!
//! ## Deferred (TODO(#198 follow-up))
//!
//! These remain recognized-but-skipped (graceful empty/partial, never a crash):
//! - mapped / conditional / utility types (`Partial<...>`, `Pick<...>`,
//!   `T extends U ? A : B`, `{ [K in Keys]: V }`),
//! - deep / nested generic substitution (only a flat type-param → arg textual
//!   substitution is performed),
//! - union props types (`A | B`) and `node_modules`-sourced prop types,
//! - re-export barrels (`export { Props } from "./x"`) for the imported type.

use std::path::{Path, PathBuf};

use tree_sitter::{Node, Parser};

/// One declared prop of a component's props type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropDef {
    /// The prop name (`primary`, `label`).
    pub name: String,
    /// The prop's TS type, as source text (`boolean`, `string`, `"sm" | "lg"`).
    pub type_text: String,
    /// Whether the prop is optional (declared with `?`).
    pub optional: bool,
}

/// Extract the ordered prop definitions of `component_name` from a component
/// source file, **without** following cross-file imports.
///
/// This is the same-file-only entry point; prefer [`extract_props_at`] when the
/// component's on-disk path is known so imported / cross-file prop types resolve.
///
/// Returns an empty vector (never an error) when the component, its props type,
/// or that type's definition can't be found — controls degrade to "no props"
/// rather than failing the whole preview.
pub fn extract_props(component_source: &str, component_name: &str) -> Vec<PropDef> {
    extract_props_at(component_source, component_name, None)
}

/// Extract the ordered prop definitions of `component_name`, optionally
/// following imports relative to `component_path` (the component's on-disk file).
///
/// When `component_path` is `Some`, an imported props type (`import type { Props
/// } from "./types"`) resolves to its sibling file. With `None`, cross-file
/// imports are skipped (same behavior as [`extract_props`]). All other shapes —
/// same-file simple, intersection of same-file operands, and generics — work in
/// both modes.
///
/// Like [`extract_props`], this never errors: any unresolved shape yields an
/// empty/partial result so controls degrade to "no props".
pub fn extract_props_at(
    component_source: &str,
    component_name: &str,
    component_path: Option<&Path>,
) -> Vec<PropDef> {
    let Some(tree) = parse_tsx(component_source) else {
        return Vec::new();
    };
    let root = tree.root_node();

    // 1. Find the component's first-parameter props type annotation.
    let Some(props_type) = find_component_props_type(component_source, root, component_name) else {
        return Vec::new();
    };

    // 2. Resolve that type to an object type and read its members.
    let resolver = Resolver::new(component_source, root, component_path);
    match props_type {
        PropsType::Inline(obj) => read_object_type_members(component_source, obj),
        PropsType::Named(name) => resolver.resolve_named(&name, &[]),
        PropsType::Generic(name, args) => resolver.resolve_named(&name, &args),
    }
}

/// Parse `source` as TSX, returning `None` if the parser can't be built or the
/// parse fails. Centralizes the language setup the entry points share.
fn parse_tsx(source: &str) -> Option<tree_sitter::Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_typescript::LANGUAGE_TSX.into())
        .ok()?;
    parser.parse(source, None)
}

/// The props type of a component: an inline object type node, the name of a type
/// to resolve elsewhere, or a generic instantiation (`Name<Arg, ...>`) carrying
/// the concrete type-argument source texts.
enum PropsType<'a> {
    Inline(Node<'a>),
    Named(String),
    Generic(String, Vec<String>),
}

/// Find the props type of `component_name`, scanning top-level declarations for
/// a matching function/const and reading its props annotation.
fn find_component_props_type<'a>(
    source: &str,
    root: Node<'a>,
    component_name: &str,
) -> Option<PropsType<'a>> {
    for child in named_children(root) {
        match child.kind() {
            // `function Button(props: ButtonProps) { ... }`
            "function_declaration" => {
                if identifier_name(source, child).as_deref() == Some(component_name) {
                    if let Some(t) = props_type_from_params(source, child) {
                        return Some(t);
                    }
                }
            }
            // `const Button = (...) => ...` / `const Button: React.FC<...> = ...`
            "lexical_declaration" | "variable_declaration" => {
                if let Some(t) = props_type_from_declarators(source, child, component_name) {
                    return Some(t);
                }
            }
            // `export function Button(...)` / `export const Button = ...` /
            // `export default function Button(...)`.
            "export_statement" => {
                for inner in named_children(child) {
                    match inner.kind() {
                        "function_declaration" => {
                            if identifier_name(source, inner).as_deref() == Some(component_name) {
                                if let Some(t) = props_type_from_params(source, inner) {
                                    return Some(t);
                                }
                            }
                        }
                        "lexical_declaration" | "variable_declaration" => {
                            if let Some(t) =
                                props_type_from_declarators(source, inner, component_name)
                            {
                                return Some(t);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Read the props type from a `const NAME = ...` declaration matching
/// `component_name`.
fn props_type_from_declarators<'a>(
    source: &str,
    decl: Node<'a>,
    component_name: &str,
) -> Option<PropsType<'a>> {
    for declarator in named_children(decl) {
        if declarator.kind() != "variable_declarator" {
            continue;
        }
        let name =
            first_child_of_kind(declarator, "identifier").map(|n| node_text(n, source).to_string());
        if name.as_deref() != Some(component_name) {
            continue;
        }

        // `const Button: React.FC<ButtonProps> = ...` — the props type is the
        // type argument of the variable's own type annotation.
        if let Some(type_ann) = first_child_of_kind(declarator, "type_annotation") {
            if let Some(arg) = fc_type_argument(source, type_ann) {
                return Some(arg);
            }
        }

        // Otherwise read the initializer's first parameter type:
        // `const Button = (props: ButtonProps) => ...`.
        for value in named_children(declarator) {
            match value.kind() {
                "arrow_function" | "function_expression" | "function" => {
                    if let Some(t) = props_type_from_params(source, value) {
                        return Some(t);
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// Extract the props type from a `React.FC<Props>` / `FC<Props>` type
/// annotation. The props type is the first type argument.
fn fc_type_argument<'a>(source: &str, type_annotation: Node<'a>) -> Option<PropsType<'a>> {
    // type_annotation -> generic_type { name, type_arguments }
    let generic = first_child_of_kind(type_annotation, "generic_type")?;
    let type_args = first_child_of_kind(generic, "type_arguments")?;
    // First named child of type_arguments is the first type argument.
    let first = named_children(type_args).into_iter().next()?;
    Some(props_type_from_type_node(source, first))
}

/// Read the props type from a callable node's parameter list (the first
/// parameter's type annotation).
fn props_type_from_params<'a>(source: &str, callable: Node<'a>) -> Option<PropsType<'a>> {
    let params = first_child_of_kind(callable, "formal_parameters")?;
    // The first required/optional parameter.
    let first_param = named_children(params)
        .into_iter()
        .find(|p| matches!(p.kind(), "required_parameter" | "optional_parameter"))?;
    let type_ann = first_child_of_kind(first_param, "type_annotation")?;
    // type_annotation's payload is its single non-trivial child.
    let type_node = named_children(type_ann).into_iter().next()?;
    Some(props_type_from_type_node(source, type_node))
}

/// Classify a type node as an inline object type, a named type reference, or a
/// generic instantiation (`Name<Arg, ...>`).
fn props_type_from_type_node<'a>(source: &str, type_node: Node<'a>) -> PropsType<'a> {
    match type_node.kind() {
        // `{ primary: boolean; ... }`
        "object_type" => PropsType::Inline(type_node),
        // `ButtonProps`
        "type_identifier" => PropsType::Named(node_text(type_node, source).to_string()),
        // `Props<Variant>` — a generic instantiation. Carry the base name plus
        // each concrete type-argument's source text for best-effort substitution.
        "generic_type" => generic_props_type(source, type_node)
            .unwrap_or_else(|| PropsType::Named(node_text(type_node, source).to_string())),
        // `Props.Whatever` (qualified) or anything else — TODO(#198 follow-up):
        // qualified / utility / mapped props types are not resolved; treat the
        // node text as a best-effort name so same-file matches still work.
        _ => PropsType::Named(node_text(type_node, source).to_string()),
    }
}

/// Decompose a `generic_type` node (`Name<Arg, ...>`) into its base type name and
/// the source text of each type argument. Returns `None` if the base name isn't a
/// plain `type_identifier` (e.g. a qualified `A.B<...>`).
fn generic_props_type<'a>(source: &str, generic: Node<'a>) -> Option<PropsType<'a>> {
    let name_node = first_child_of_kind(generic, "type_identifier")?;
    let name = node_text(name_node, source).to_string();
    let args = match first_child_of_kind(generic, "type_arguments") {
        Some(type_args) => named_children(type_args)
            .into_iter()
            .map(|a| node_text(a, source).trim().to_string())
            .collect(),
        None => Vec::new(),
    };
    Some(PropsType::Generic(name, args))
}

/// Resolves a named props type to its member [`PropDef`]s, following same-file
/// interface/type-alias declarations, intersections of those, generic
/// substitution, and (when a file path is known) cross-file imports.
///
/// One resolver is scoped to a single component source + its root node + its
/// on-disk path; cross-file hops re-parse the imported sibling and recurse into a
/// fresh resolver anchored at *that* file (so its own relative imports resolve).
struct Resolver<'a> {
    source: &'a str,
    root: Node<'a>,
    /// The component file's own path, used to resolve relative imports. `None`
    /// disables cross-file resolution.
    path: Option<PathBuf>,
}

impl<'a> Resolver<'a> {
    fn new(source: &'a str, root: Node<'a>, path: Option<&Path>) -> Self {
        Self {
            source,
            root,
            path: path.map(Path::to_path_buf),
        }
    }

    /// Resolve `type_name` (optionally with generic `type_args`) to its props.
    ///
    /// Tries, in order: a same-file interface/type-alias declaration; then — if
    /// unresolved and a path is known — an imported sibling declaration. Returns
    /// an empty vector when nothing resolves.
    fn resolve_named(&self, type_name: &str, type_args: &[String]) -> Vec<PropDef> {
        if let Some(decl) = self.find_local_decl(type_name) {
            return self.props_from_decl(self.source, decl, type_args);
        }
        // Cross-file: the name is imported from a sibling file.
        if self.path.is_some() {
            if let Some(props) = self.resolve_imported(type_name, type_args) {
                return props;
            }
        }
        Vec::new()
    }

    /// Find a top-level `interface`/`type` declaration named `type_name` in this
    /// resolver's source (including `export`-wrapped declarations).
    fn find_local_decl(&self, type_name: &str) -> Option<Node<'a>> {
        find_type_decl(self.source, self.root, type_name)
    }

    /// Read the props of a resolved `interface`/`type_alias` declaration,
    /// substituting `type_args` for the declaration's generic parameters.
    fn props_from_decl(&self, source: &str, decl: Node, type_args: &[String]) -> Vec<PropDef> {
        let subst = type_param_substitution(source, decl, type_args);
        match decl.kind() {
            "interface_declaration" => {
                // `interface Name<...> extends Base { ... }` — read the body and,
                // best-effort, fold in any extended bases that resolve locally.
                let mut out = Vec::new();
                if let Some(body) = first_child_of_kind(decl, "interface_body")
                    .or_else(|| first_child_of_kind(decl, "object_type"))
                {
                    out = read_object_type_members_subst(source, body, &subst);
                }
                for base in interface_extends_names(source, decl) {
                    merge_props(&mut out, self.resolve_named(&base, &[]));
                }
                out
            }
            "type_alias_declaration" => self.props_from_type_alias(source, decl, &subst),
            _ => Vec::new(),
        }
    }

    /// Read the props of a `type Name = <rhs>` alias: a direct object type, or an
    /// intersection whose operands are each resolved + unioned.
    fn props_from_type_alias(
        &self,
        source: &str,
        decl: Node,
        subst: &[(String, String)],
    ) -> Vec<PropDef> {
        // The rhs is the alias's `value` field (its last non-type-parameter child).
        let Some(rhs) = type_alias_rhs(decl) else {
            return Vec::new();
        };
        match rhs.kind() {
            // `type P = { ... }`
            "object_type" => read_object_type_members_subst(source, rhs, subst),
            // `type P = Base & Extra & { inline: ... }`
            "intersection_type" => self.props_from_intersection(source, rhs, subst),
            // `type P = Other` / `type P = Other<Arg>` — alias to another type.
            "type_identifier" => self.resolve_named(node_text(rhs, source), &[]),
            "generic_type" => match generic_props_type(source, rhs) {
                Some(PropsType::Generic(name, args)) => self.resolve_named(&name, &args),
                _ => Vec::new(),
            },
            // TODO(#198 follow-up): unions / conditional / mapped / utility-type
            // rhs (`A | B`, `Partial<...>`, `{ [K in Keys]: V }`) are not
            // destructured — graceful empty.
            _ => Vec::new(),
        }
    }

    /// Union the members of every operand of an intersection (`A & B & {..}`).
    ///
    /// Each operand is resolved (a same-file/imported named type, an inline
    /// object literal, or a nested generic) and the results are merged, first
    /// operand winning on a prop-name clash.
    fn props_from_intersection(
        &self,
        source: &str,
        intersection: Node,
        subst: &[(String, String)],
    ) -> Vec<PropDef> {
        let mut out: Vec<PropDef> = Vec::new();
        for operand in named_children(intersection) {
            let operand_props = match operand.kind() {
                "object_type" => read_object_type_members_subst(source, operand, subst),
                "type_identifier" => self.resolve_named(node_text(operand, source), &[]),
                "generic_type" => match generic_props_type(source, operand) {
                    Some(PropsType::Generic(name, args)) => self.resolve_named(&name, &args),
                    _ => Vec::new(),
                },
                // Nested intersection (rare) — recurse.
                "intersection_type" => self.props_from_intersection(source, operand, subst),
                // TODO(#198 follow-up): union operands inside an intersection are
                // not destructured.
                _ => Vec::new(),
            };
            merge_props(&mut out, operand_props);
        }
        out
    }

    /// Resolve `type_name` via a sibling file it is imported from.
    ///
    /// Finds an `import`/`import type` statement that binds `type_name` and has a
    /// **relative** specifier, resolves the sibling file next to this resolver's
    /// path, re-parses it, and resolves `type_name` (with `type_args`) there.
    /// Returns `None` for bare/`node_modules` specifiers or unresolvable paths.
    fn resolve_imported(&self, type_name: &str, type_args: &[String]) -> Option<Vec<PropDef>> {
        let component_path = self.path.as_ref()?;
        let specifier = import_specifier_for(self.source, self.root, type_name)?;
        // Project-local relative imports only; never chase node_modules / bare.
        if !specifier.starts_with('.') {
            return None;
        }
        let base_dir = component_path.parent()?;
        let resolved = resolve_relative_type_file(base_dir, &specifier)?;
        let imported_source = std::fs::read_to_string(&resolved).ok()?;
        let tree = parse_tsx(&imported_source)?;
        let imported_root = tree.root_node();
        let inner = Resolver::new(&imported_source, imported_root, Some(&resolved));
        Some(inner.resolve_named(type_name, type_args))
    }
}

/// Find a top-level `interface`/`type` declaration named `type_name` in `root`'s
/// source, unwrapping a leading `export`.
fn find_type_decl<'a>(source: &str, root: Node<'a>, type_name: &str) -> Option<Node<'a>> {
    for child in named_children(root) {
        let decl = match child.kind() {
            "interface_declaration" | "type_alias_declaration" => child,
            "export_statement" => match named_children(child)
                .into_iter()
                .find(|n| matches!(n.kind(), "interface_declaration" | "type_alias_declaration"))
            {
                Some(d) => d,
                None => continue,
            },
            _ => continue,
        };
        let name =
            first_child_of_kind(decl, "type_identifier").map(|n| node_text(n, source).to_string());
        if name.as_deref() == Some(type_name) {
            return Some(decl);
        }
    }
    None
}

/// The rhs (`value`) node of a `type X = <rhs>` alias: its last named child after
/// the name and any `type_parameters`.
fn type_alias_rhs(decl: Node) -> Option<Node> {
    named_children(decl)
        .into_iter()
        .rev()
        .find(|n| !matches!(n.kind(), "type_identifier" | "type_parameters"))
}

/// The base type names of an `interface X extends A, B { ... }` clause.
///
/// Best-effort: only plain `type_identifier` bases are returned (generic /
/// qualified bases are skipped — TODO(#198 follow-up)).
fn interface_extends_names(source: &str, decl: Node) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(clause) = first_child_of_kind(decl, "extends_type_clause") {
        for base in named_children(clause) {
            if base.kind() == "type_identifier" {
                out.push(node_text(base, source).to_string());
            }
        }
    }
    out
}

/// Build the `[(param, arg)]` substitution map for a generic declaration, pairing
/// each declared `type_parameter` name with the concrete `type_args` text by
/// position. Extra params (no matching arg) are dropped.
fn type_param_substitution(
    source: &str,
    decl: Node,
    type_args: &[String],
) -> Vec<(String, String)> {
    if type_args.is_empty() {
        return Vec::new();
    }
    let Some(params) = first_child_of_kind(decl, "type_parameters") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (param, arg) in named_children(params)
        .into_iter()
        .filter(|p| p.kind() == "type_parameter")
        .zip(type_args)
    {
        if let Some(name) = first_child_of_kind(param, "type_identifier") {
            out.push((node_text(name, source).to_string(), arg.clone()));
        }
    }
    out
}

/// Merge `incoming` props into `out`, skipping any whose name already exists
/// (first occurrence wins — left-to-right declaration / intersection order).
fn merge_props(out: &mut Vec<PropDef>, incoming: Vec<PropDef>) {
    for prop in incoming {
        if !out.iter().any(|p| p.name == prop.name) {
            out.push(prop);
        }
    }
}

/// Resolve a relative module specifier to an existing type-bearing file under
/// `base_dir`, probing TS/JS extensions, `.d.ts`, and an `index.*` barrel.
fn resolve_relative_type_file(base_dir: &Path, specifier: &str) -> Option<PathBuf> {
    let joined = base_dir.join(specifier);
    if joined.is_file() {
        return Some(joined);
    }
    const EXTS: &[&str] = &["ts", "tsx", "d.ts", "jsx", "js"];
    for ext in EXTS {
        let candidate = joined.with_extension(ext);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    for ext in EXTS {
        let candidate = joined.join(format!("index.{ext}"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Find the relative import specifier that binds the type name `type_name` in
/// `root`'s source (`import type { Props } from "./types"`,
/// `import { Props } from "./types"`, `import { Foo as Props } ...`). Returns the
/// specifier (`./types`) of the first matching import.
fn import_specifier_for(source: &str, root: Node, type_name: &str) -> Option<String> {
    for child in named_children(root) {
        if child.kind() != "import_statement" {
            continue;
        }
        let import_text = node_text(child, source);
        if !import_binds_type(import_text, type_name) {
            continue;
        }
        // The specifier is the import's (last) string child.
        let source_node = {
            let mut c = child.walk();
            child
                .named_children(&mut c)
                .filter(|n| n.kind() == "string")
                .last()
        }?;
        let raw = node_text(source_node, source);
        return Some(
            raw.trim_matches(|q| q == '"' || q == '\'' || q == '`')
                .to_string(),
        );
    }
    None
}

/// True when an import statement's text binds the local type name `name` as a
/// named (possibly aliased) import. Default/namespace imports of a *type* are not
/// valid TS, so only the `{ ... }` clause is inspected.
fn import_binds_type(import_text: &str, name: &str) -> bool {
    let Some(open) = import_text.find('{') else {
        return false;
    };
    let Some(close_rel) = import_text[open..].find('}') else {
        return false;
    };
    let inner = &import_text[open + 1..open + close_rel];
    for spec in inner.split(',') {
        // The local binding is the token after `as`, else the token itself; drop
        // a leading per-specifier `type` keyword (`import { type Props }`).
        let local = spec
            .rsplit(" as ")
            .next()
            .unwrap_or(spec)
            .trim()
            .trim_start_matches("type ")
            .trim();
        if local == name {
            return true;
        }
    }
    false
}

/// Read the `name: type` members of a TS object/interface body in source order.
///
/// Each `property_signature` contributes a [`PropDef`]; the `?` token marks the
/// prop optional. Index signatures, method signatures, and spreads are skipped.
fn read_object_type_members(source: &str, body: Node) -> Vec<PropDef> {
    read_object_type_members_subst(source, body, &[])
}

/// Like [`read_object_type_members`], but textually substitutes each
/// `(param, arg)` in `subst` for a generic type parameter referenced inside a
/// member's type text. Best-effort: only whole-word identifier occurrences of a
/// param are replaced (so `T` → `Variant`, but `Theme` is untouched).
fn read_object_type_members_subst(
    source: &str,
    body: Node,
    subst: &[(String, String)],
) -> Vec<PropDef> {
    let mut out = Vec::new();
    for member in named_children(body) {
        if member.kind() != "property_signature" {
            // TODO(#198 follow-up): method_signature / index_signature /
            // call_signature members are not surfaced as props.
            continue;
        }
        let Some(name_node) = first_child_of_kind(member, "property_identifier") else {
            continue;
        };
        let name = node_text(name_node, source).to_string();
        let optional = member_is_optional(member);
        let raw_type = first_child_of_kind(member, "type_annotation")
            .and_then(|ann| named_children(ann).into_iter().next())
            .map(|t| node_text(t, source).trim().to_string())
            .unwrap_or_default();
        let type_text = apply_type_param_subst(&raw_type, subst);
        out.push(PropDef {
            name,
            type_text,
            optional,
        });
    }
    out
}

/// Substitute each generic `(param, arg)` for whole-word occurrences of `param`
/// in `type_text`. A whole word is a maximal run of identifier characters
/// (`[A-Za-z0-9_$]`), so `T` is replaced but `Theme`/`xT` are not.
fn apply_type_param_subst(type_text: &str, subst: &[(String, String)]) -> String {
    if subst.is_empty() {
        return type_text.to_string();
    }
    let is_ident = |c: char| c.is_alphanumeric() || c == '_' || c == '$';
    let mut out = String::with_capacity(type_text.len());
    let mut token = String::new();
    let flush = |token: &mut String, out: &mut String| {
        if token.is_empty() {
            return;
        }
        let replacement = subst
            .iter()
            .find(|(param, _)| param == token)
            .map(|(_, arg)| arg.as_str())
            .unwrap_or(token.as_str());
        out.push_str(replacement);
        token.clear();
    };
    for c in type_text.chars() {
        if is_ident(c) {
            token.push(c);
        } else {
            flush(&mut token, &mut out);
            out.push(c);
        }
    }
    flush(&mut token, &mut out);
    out
}

/// A property signature is optional when it contains a `?` token between the
/// name and the type annotation. tree-sitter exposes it as an anonymous `?`
/// child, so scan the member's raw children for it.
fn member_is_optional(member: Node) -> bool {
    let mut cursor = member.walk();
    let optional = member.children(&mut cursor).any(|c| c.kind() == "?");
    optional
}

// ── AST helpers (mirrors the small set used in csf.rs / frontend.rs) ──────────

fn named_children(node: Node) -> Vec<Node> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).collect()
}

fn first_child_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    named_children(node).into_iter().find(|c| c.kind() == kind)
}

fn identifier_name(source: &str, node: Node) -> Option<String> {
    first_child_of_kind(node, "identifier").map(|n| node_text(n, source).to_string())
}

fn node_text<'a>(node: Node, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUTTON: &str = r#"
import React from 'react';

interface ButtonProps {
  primary: boolean;
  label: string;
  size: "sm" | "lg";
  count?: number;
}

export function Button(props: ButtonProps) {
  return null;
}
"#;

    #[test]
    fn extracts_interface_props_in_order() {
        let props = extract_props(BUTTON, "Button");
        let names: Vec<&str> = props.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["primary", "label", "size", "count"]);
        assert_eq!(props[0].type_text, "boolean");
        assert_eq!(props[1].type_text, "string");
        assert_eq!(props[2].type_text, "\"sm\" | \"lg\"");
        assert_eq!(props[3].type_text, "number");
        assert!(!props[0].optional);
        assert!(props[3].optional, "count? is optional");
    }

    #[test]
    fn extracts_from_arrow_const() {
        let src = r#"
type CardProps = { title: string; elevated: boolean };
export const Card = (props: CardProps) => null;
"#;
        let props = extract_props(src, "Card");
        let names: Vec<&str> = props.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["title", "elevated"]);
    }

    #[test]
    fn extracts_from_react_fc() {
        let src = r#"
import React from 'react';
interface BadgeProps { text: string; }
const Badge: React.FC<BadgeProps> = (props) => null;
"#;
        let props = extract_props(src, "Badge");
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].name, "text");
        assert_eq!(props[0].type_text, "string");
    }

    #[test]
    fn extracts_inline_object_type() {
        let src = r#"
export function Tag(props: { name: string; closable?: boolean }) { return null; }
"#;
        let props = extract_props(src, "Tag");
        let names: Vec<&str> = props.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["name", "closable"]);
        assert!(props[1].optional);
    }

    #[test]
    fn unknown_component_yields_empty() {
        assert!(extract_props(BUTTON, "Missing").is_empty());
    }

    // ── #198: intersection / generic / extends (same-file, no path) ───────────

    #[test]
    fn intersection_unions_local_operands() {
        let src = r#"
interface Base { id: string; primary: boolean; }
type Extra = { count: number; label: string; };
type Props = Base & Extra;
export function Widget(props: Props) { return null; }
"#;
        let props = extract_props(src, "Widget");
        let names: Vec<&str> = props.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["id", "primary", "count", "label"]);
        assert_eq!(props[2].type_text, "number");
    }

    #[test]
    fn intersection_first_operand_wins_on_clash() {
        let src = r#"
interface Base { size: "sm" | "lg"; }
type Extra = { size: number; };
type Props = Base & Extra;
export const Box = (props: Props) => null;
"#;
        let props = extract_props(src, "Box");
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].name, "size");
        assert_eq!(props[0].type_text, "\"sm\" | \"lg\"", "first operand wins");
    }

    #[test]
    fn generic_substitutes_type_argument() {
        let src = r#"
interface GenProps<T> { value: T; label: string; }
export function Field(props: GenProps<number>) { return null; }
"#;
        let props = extract_props(src, "Field");
        assert_eq!(props.len(), 2);
        assert_eq!(props[0].name, "value");
        assert_eq!(props[0].type_text, "number", "T substituted with number");
        assert_eq!(props[1].type_text, "string");
    }

    #[test]
    fn generic_react_fc_with_type_arg() {
        let src = r#"
import React from 'react';
interface GenProps<T> { value: T; }
const Field: React.FC<GenProps<boolean>> = (props) => null;
"#;
        let props = extract_props(src, "Field");
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].type_text, "boolean");
    }

    #[test]
    fn generic_without_arg_reads_unparameterized_members() {
        // No type argument supplied at the use site → members read verbatim
        // (the param-typed member keeps its `T` text, falls back to Text control).
        let src = r#"
interface GenProps<T> { value: T; label: string; }
export function Field(props: GenProps) { return null; }
"#;
        let props = extract_props(src, "Field");
        let names: Vec<&str> = props.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["value", "label"]);
        assert_eq!(props[0].type_text, "T");
    }

    #[test]
    fn interface_extends_folds_in_base_members() {
        let src = r#"
interface Base { id: string; }
interface Props extends Base { label: string; }
export function Widget(props: Props) { return null; }
"#;
        let props = extract_props(src, "Widget");
        let names: Vec<&str> = props.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["label", "id"]);
    }

    #[test]
    fn unresolvable_generic_degrades_to_empty() {
        // A utility/mapped-type rhs we don't destructure → graceful empty.
        let src = r#"
type Props = Partial<{ a: string }>;
export function Widget(props: Props) { return null; }
"#;
        assert!(extract_props(src, "Widget").is_empty());
    }

    #[test]
    fn imported_props_resolve_from_sibling_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let root = dir.path();
        std::fs::write(
            root.join("types.ts"),
            "export interface Props { id: string; active: boolean; }\n",
        )
        .unwrap();
        let component = root.join("Widget.tsx");
        std::fs::write(
            &component,
            r#"
import type { Props } from "./types";
export function Widget(props: Props) { return null; }
"#,
        )
        .unwrap();

        let source = std::fs::read_to_string(&component).unwrap();
        let props = extract_props_at(&source, "Widget", Some(&component));
        let names: Vec<&str> = props.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["id", "active"]);
    }

    #[test]
    fn imported_props_without_path_yield_empty() {
        // Same source, but no path → cross-file import is not followed.
        let source = r#"
import type { Props } from "./types";
export function Widget(props: Props) { return null; }
"#;
        assert!(extract_props(source, "Widget").is_empty());
    }
}
// </HANDWRITE>

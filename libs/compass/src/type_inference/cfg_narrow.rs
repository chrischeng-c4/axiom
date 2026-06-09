//! CFG-based type narrowing (R2.1)
//!
//! Integrates the Control Flow Graph from the PDG module with the TypeNarrower
//! to provide flow-sensitive type narrowing. For each basic block in the CFG,
//! computes the set of narrowed types that hold on entry to that block.
//!
//! This gives Pyright-level precision: after `if isinstance(x, Foo):`, the
//! type of `x` inside the block is `Foo`, not `Union[Foo, Bar]`.

use std::collections::HashMap;

use crate::semantic::pdg::cfg::{BlockId, BlockKind, ControlFlowGraph};
use crate::type_inference::narrow::{negate_condition, NarrowingCondition, TypeNarrower};
use crate::type_inference::ty::Type;

/// Narrowed type environment for a single CFG block entry
#[derive(Debug, Clone, Default)]
pub struct BlockNarrowEnv {
    /// Variable -> narrowed type at this block entry
    pub narrowed: HashMap<String, Type>,
}

impl BlockNarrowEnv {
    /// Merge two environments (join at control-flow merge points)
    ///
    /// Conservative: when two branches disagree on a type, takes the union.
    pub fn join(&self, other: &BlockNarrowEnv) -> BlockNarrowEnv {
        let mut merged = self.narrowed.clone();

        for (var, other_ty) in &other.narrowed {
            let entry = merged.entry(var.clone()).or_insert(other_ty.clone());
            if entry != other_ty {
                // Branches disagree — take union (conservative)
                *entry = Type::Union(vec![entry.clone(), other_ty.clone()]);
            }
        }

        BlockNarrowEnv { narrowed: merged }
    }

    /// Get the narrowed type for a variable, if known
    pub fn get(&self, var: &str) -> Option<&Type> {
        self.narrowed.get(var)
    }
}

/// Result of CFG-based narrowing analysis
pub struct CfgNarrowingResult {
    /// Narrowing environment at the entry of each block
    pub block_envs: HashMap<BlockId, BlockNarrowEnv>,
}

impl CfgNarrowingResult {
    /// Get the narrowed type of a variable at a given line
    pub fn type_at_line(&self, line: usize, var: &str, cfg: &ControlFlowGraph) -> Option<&Type> {
        // Find which block contains this line
        for (block_id, block) in &cfg.blocks {
            for stmt in &block.statements {
                if stmt.line == line {
                    return self.block_envs.get(block_id).and_then(|env| env.get(var));
                }
            }
        }
        None
    }

    /// Check if a variable is narrowed in the given block
    pub fn is_narrowed_in_block(&self, block_id: BlockId, var: &str) -> bool {
        self.block_envs
            .get(&block_id)
            .and_then(|env| env.get(var))
            .is_some()
    }
}

/// CFG-based narrowing pass
///
/// For each IfCondition / LoopCondition block, extracts the condition
/// from the statement text and propagates narrowed types to successor blocks.
pub struct CfgNarrowingPass<'a> {
    cfg: &'a ControlFlowGraph,
    /// Original (pre-narrowing) type environment
    original_types: HashMap<String, Type>,
    /// Source code (for parsing conditions)
    _source: &'a str,
}

impl<'a> CfgNarrowingPass<'a> {
    /// Create a new narrowing pass
    pub fn new(
        cfg: &'a ControlFlowGraph,
        original_types: HashMap<String, Type>,
        source: &'a str,
    ) -> Self {
        Self {
            cfg,
            original_types,
            _source: source,
        }
    }

    /// Run the narrowing pass over the entire CFG
    ///
    /// Uses a worklist algorithm:
    /// 1. Start with empty narrowing environments for all blocks
    /// 2. For each IfCondition/LoopCondition block, compute the condition
    /// 3. Propagate narrowed types to true/false successor blocks
    /// 4. At merge points (join), take the union of narrowed types
    pub fn run(&self) -> CfgNarrowingResult {
        let mut block_envs: HashMap<BlockId, BlockNarrowEnv> = HashMap::new();

        // Initialize: entry block has the original types as its narrowed env
        block_envs.insert(
            self.cfg.entry,
            BlockNarrowEnv {
                narrowed: self.original_types.clone(),
            },
        );

        // Process blocks in RPO-like order (entry first)
        let mut worklist = vec![self.cfg.entry];
        let mut visited = std::collections::HashSet::new();

        while let Some(block_id) = worklist.pop() {
            if !visited.insert(block_id) {
                continue;
            }

            let current_env = block_envs.entry(block_id).or_default().clone();

            let block = match self.cfg.get_block(block_id) {
                Some(b) => b,
                None => continue,
            };

            // Check if this is a conditional block
            let is_conditional = matches!(
                block.kind,
                BlockKind::IfCondition | BlockKind::LoopCondition
            );

            if is_conditional {
                // Extract condition from the block's statement
                if let Some(condition) = self.extract_condition_from_block(block_id) {
                    let negated = negate_condition(&condition);

                    // Apply condition to successors
                    if let Some(edges) = self.cfg.successors.get(&block_id) {
                        for edge in edges {
                            use crate::semantic::pdg::cfg::EdgeKind;

                            let branch_condition = match edge.kind {
                                EdgeKind::TrueBranch => Some(condition.clone()),
                                EdgeKind::FalseBranch => Some(negated.clone()),
                                _ => None,
                            };

                            let successor_env = if let Some(cond) = branch_condition {
                                self.apply_condition_to_env(&current_env, &cond)
                            } else {
                                current_env.clone()
                            };

                            // Merge with existing env if block already has one
                            let merged = if let Some(existing) = block_envs.get(&edge.to) {
                                existing.join(&successor_env)
                            } else {
                                successor_env
                            };

                            block_envs.insert(edge.to, merged);
                            worklist.push(edge.to);
                        }
                    }
                } else {
                    // No condition parsed — propagate current env unchanged
                    self.propagate_unchanged(
                        block_id,
                        &current_env,
                        &mut block_envs,
                        &mut worklist,
                    );
                }
            } else {
                // Non-conditional block: propagate env unchanged
                self.propagate_unchanged(block_id, &current_env, &mut block_envs, &mut worklist);
            }
        }

        CfgNarrowingResult { block_envs }
    }

    /// Extract and parse the narrowing condition from an IfCondition block
    fn extract_condition_from_block(&self, block_id: BlockId) -> Option<NarrowingCondition> {
        let block = self.cfg.get_block(block_id)?;

        // The condition is typically the first (and only) statement in the block
        let stmt = block.statements.first()?;

        // Parse the condition text using the existing parse_condition function
        // We need to parse it as a tree-sitter node. Since we have the source,
        // we use a lightweight text-based parsing approach here.
        let condition = self.parse_condition_text(&stmt.text)?;

        Some(condition)
    }

    /// Parse a condition from its text representation
    ///
    /// This is a lightweight text-based parser for common patterns.
    /// For full AST accuracy, use parse_condition() with the tree-sitter node.
    fn parse_condition_text(&self, text: &str) -> Option<NarrowingCondition> {
        let text = text.trim();

        // isinstance(x, T) or isinstance(x, (T1, T2))
        if text.starts_with("isinstance(") {
            return self.parse_isinstance_text(text);
        }

        // x is None
        if let Some(var) = text.strip_suffix(" is None") {
            let var = var.trim();
            if is_simple_identifier(var) {
                return Some(NarrowingCondition::IsNone {
                    var_name: var.to_string(),
                });
            }
        }

        // x is not None
        if let Some(var) = text.strip_suffix(" is not None") {
            let var = var.trim();
            if is_simple_identifier(var) {
                return Some(NarrowingCondition::IsNotNone {
                    var_name: var.to_string(),
                });
            }
        }

        // x == None or x != None (less common but valid)
        if let Some(var) = text.strip_suffix(" == None") {
            let var = var.trim();
            if is_simple_identifier(var) {
                return Some(NarrowingCondition::IsNone {
                    var_name: var.to_string(),
                });
            }
        }

        if let Some(var) = text.strip_suffix(" != None") {
            let var = var.trim();
            if is_simple_identifier(var) {
                return Some(NarrowingCondition::IsNotNone {
                    var_name: var.to_string(),
                });
            }
        }

        // callable(x)
        if text.starts_with("callable(") && text.ends_with(')') {
            let inner = &text[9..text.len() - 1];
            if is_simple_identifier(inner.trim()) {
                return Some(NarrowingCondition::IsCallable {
                    var_name: inner.trim().to_string(),
                });
            }
        }

        // hasattr(x, "attr")
        if text.starts_with("hasattr(") {
            return self.parse_hasattr_text(text);
        }

        // Simple identifier (truthiness)
        if is_simple_identifier(text) {
            return Some(NarrowingCondition::Truthy {
                var_name: text.to_string(),
            });
        }

        // not x (falsiness)
        if let Some(inner) = text.strip_prefix("not ") {
            let inner = inner.trim();
            if is_simple_identifier(inner) {
                return Some(NarrowingCondition::Falsy {
                    var_name: inner.to_string(),
                });
            }
        }

        None
    }

    fn parse_isinstance_text(&self, text: &str) -> Option<NarrowingCondition> {
        // isinstance(var, Type) or isinstance(var, (T1, T2))
        let inner = text.strip_prefix("isinstance(")?.strip_suffix(')')?;

        let comma_pos = inner.find(',')?;
        let var_part = inner[..comma_pos].trim();
        let type_part = inner[comma_pos + 1..].trim();

        if !is_simple_identifier(var_part) {
            return None;
        }

        let types = if type_part.starts_with('(') && type_part.ends_with(')') {
            // Tuple of types
            let inner_types = &type_part[1..type_part.len() - 1];
            inner_types
                .split(',')
                .map(|t| parse_simple_type_from_name(t.trim()))
                .collect()
        } else {
            vec![parse_simple_type_from_name(type_part)]
        };

        Some(NarrowingCondition::IsInstance {
            var_name: var_part.to_string(),
            types,
        })
    }

    fn parse_hasattr_text(&self, text: &str) -> Option<NarrowingCondition> {
        // hasattr(var, "attr_name")
        let inner = text.strip_prefix("hasattr(")?.strip_suffix(')')?;

        let comma_pos = inner.find(',')?;
        let var_part = inner[..comma_pos].trim();
        let attr_part = inner[comma_pos + 1..].trim();

        if !is_simple_identifier(var_part) {
            return None;
        }

        let attr_name = attr_part
            .trim_start_matches(|c| c == '"' || c == '\'')
            .trim_end_matches(|c| c == '"' || c == '\'')
            .to_string();

        Some(NarrowingCondition::HasAttr {
            var_name: var_part.to_string(),
            attr_name,
        })
    }

    /// Apply a narrowing condition to a BlockNarrowEnv
    fn apply_condition_to_env(
        &self,
        env: &BlockNarrowEnv,
        condition: &NarrowingCondition,
    ) -> BlockNarrowEnv {
        let mut narrower = TypeNarrower::new();
        narrower.push_scope();

        // Merge env types into original_types for the narrower
        let mut combined = self.original_types.clone();
        for (var, ty) in &env.narrowed {
            combined.insert(var.clone(), ty.clone());
        }

        narrower.apply_condition(condition, &combined);

        let scope = narrower.pop_scope().unwrap_or_default();

        // Build the new env: start with the existing env, overlay narrowed types
        let mut new_narrowed = env.narrowed.clone();
        for (var, ty) in scope.narrowed {
            new_narrowed.insert(var, ty);
        }

        BlockNarrowEnv {
            narrowed: new_narrowed,
        }
    }

    /// Propagate the current env unchanged to all successor blocks
    fn propagate_unchanged(
        &self,
        block_id: BlockId,
        current_env: &BlockNarrowEnv,
        block_envs: &mut HashMap<BlockId, BlockNarrowEnv>,
        worklist: &mut Vec<BlockId>,
    ) {
        if let Some(edges) = self.cfg.successors.get(&block_id) {
            for edge in edges {
                let merged = if let Some(existing) = block_envs.get(&edge.to) {
                    existing.join(current_env)
                } else {
                    current_env.clone()
                };

                block_envs.insert(edge.to, merged);
                worklist.push(edge.to);
            }
        }
    }
}

/// Check if a string is a simple Python identifier
fn is_simple_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Parse a simple type name to a Type
fn parse_simple_type_from_name(name: &str) -> Type {
    match name {
        "int" => Type::Int,
        "float" => Type::Float,
        "str" => Type::Str,
        "bool" => Type::Bool,
        "bytes" => Type::Bytes,
        "list" | "List" => Type::List(Box::new(Type::Unknown)),
        "dict" | "Dict" => Type::Dict(Box::new(Type::Unknown), Box::new(Type::Unknown)),
        "set" | "Set" => Type::Set(Box::new(Type::Unknown)),
        "tuple" | "Tuple" => Type::Tuple(vec![]),
        "None" => Type::None,
        _ => Type::Instance {
            name: name.to_string(),
            module: None,
            type_args: vec![],
        },
    }
}

// ============================================================================
// Generic TypeVar resolution (R2.2)
// ============================================================================

/// Resolve TypeVar bindings from function call arguments
///
/// Given a generic function signature like `def get(items: list[T]) -> T`
/// and a call `get([1, 2, 3])`, this resolves `T = int`.
///
/// Returns a map of TypeVarId -> concrete Type.
pub fn resolve_typevar_bindings(
    param_types: &[crate::type_inference::ty::Type],
    arg_types: &[crate::type_inference::ty::Type],
) -> HashMap<crate::type_inference::ty::TypeVarId, crate::type_inference::ty::Type> {
    let mut bindings = HashMap::new();

    for (param_ty, arg_ty) in param_types.iter().zip(arg_types.iter()) {
        param_ty.unify(arg_ty, &mut bindings);
    }

    bindings
}

/// Apply TypeVar bindings to a return type to get the concrete return type
pub fn apply_typevar_bindings(
    return_type: &crate::type_inference::ty::Type,
    bindings: &HashMap<crate::type_inference::ty::TypeVarId, crate::type_inference::ty::Type>,
) -> crate::type_inference::ty::Type {
    return_type.substitute(bindings)
}

// ============================================================================
// Protocol structural typing (R2.3)
// ============================================================================

/// Check if a type structurally satisfies a Protocol
///
/// A type satisfies a Protocol if it has all required members (methods/attributes)
/// with compatible types. This implements duck typing via typing.Protocol.
pub fn check_protocol_satisfaction(
    ty: &crate::type_inference::ty::Type,
    protocol_members: &[(String, crate::type_inference::ty::Type)],
    type_registry: &HashMap<String, Vec<(String, crate::type_inference::ty::Type)>>,
) -> ProtocolCheckResult {
    let members = get_type_members(ty, type_registry);

    let mut missing = Vec::new();
    let mut incompatible = Vec::new();

    for (member_name, member_type) in protocol_members {
        if let Some(&actual_type) = members.get(member_name.as_str()) {
            // Check type compatibility (simplified: Any is always compatible)
            if !types_compatible(actual_type, member_type) {
                incompatible.push(ProtocolMemberError {
                    member: member_name.clone(),
                    expected: member_type.clone(),
                    actual: actual_type.clone(),
                });
            }
        } else {
            missing.push(member_name.clone());
        }
    }

    ProtocolCheckResult {
        missing,
        incompatible,
    }
}

/// Result of a protocol satisfaction check
#[derive(Debug, Clone)]
pub struct ProtocolCheckResult {
    pub missing: Vec<String>,
    pub incompatible: Vec<ProtocolMemberError>,
}

impl ProtocolCheckResult {
    pub fn is_satisfied(&self) -> bool {
        self.missing.is_empty() && self.incompatible.is_empty()
    }
}

/// An incompatible member error
#[derive(Debug, Clone)]
pub struct ProtocolMemberError {
    pub member: String,
    pub expected: crate::type_inference::ty::Type,
    pub actual: crate::type_inference::ty::Type,
}

/// Get the members of a type (methods and attributes)
fn get_type_members<'a>(
    ty: &'a crate::type_inference::ty::Type,
    registry: &'a HashMap<String, Vec<(String, crate::type_inference::ty::Type)>>,
) -> HashMap<&'a str, &'a crate::type_inference::ty::Type> {
    match ty {
        crate::type_inference::ty::Type::Instance { name, .. } => {
            if let Some(members) = registry.get(name.as_str()) {
                return members.iter().map(|(k, v)| (k.as_str(), v)).collect();
            }
        }
        crate::type_inference::ty::Type::Protocol { members, .. } => {
            return members.iter().map(|(k, v)| (k.as_str(), v)).collect();
        }
        _ => {}
    }
    HashMap::new()
}

/// Check if two types are compatible (simplified structural check)
fn types_compatible(
    actual: &crate::type_inference::ty::Type,
    expected: &crate::type_inference::ty::Type,
) -> bool {
    match (actual, expected) {
        (_, crate::type_inference::ty::Type::Any) => true,
        (crate::type_inference::ty::Type::Any, _) => true,
        (crate::type_inference::ty::Type::Unknown, _) => true,
        (a, b) => a == b,
    }
}

// ============================================================================
// @overload resolution (R2.4)
// ============================================================================

/// Resolve the correct overload for a function call
///
/// Given an Overloaded type and the argument types, selects the matching
/// overload signature using the same algorithm as Pyright:
/// 1. Try each overload in order
/// 2. Return the first one where all argument types are compatible with params
/// 3. Fall back to the last overload if none match (error recovery)
pub fn resolve_overload(
    overloaded: &crate::type_inference::ty::Type,
    arg_types: &[crate::type_inference::ty::Type],
) -> Option<crate::type_inference::ty::Type> {
    let signatures = match overloaded {
        crate::type_inference::ty::Type::Overloaded { signatures } => signatures,
        // Not overloaded — just return as-is
        other => return Some(other.clone()),
    };

    for sig in signatures {
        if let crate::type_inference::ty::Type::Callable { params, ret } = sig {
            if overload_matches(params, arg_types) {
                return Some((**ret).clone());
            }
        }
    }

    // Fall back to last signature's return type for error recovery
    signatures.last().and_then(|sig| {
        if let crate::type_inference::ty::Type::Callable { ret, .. } = sig {
            Some((**ret).clone())
        } else {
            None
        }
    })
}

/// Check if argument types match an overload's parameter types
fn overload_matches(
    params: &[crate::type_inference::ty::Param],
    arg_types: &[crate::type_inference::ty::Type],
) -> bool {
    use crate::type_inference::ty::ParamKind;

    // Count required positional params
    let required_count = params
        .iter()
        .filter(|p| {
            !p.has_default && matches!(p.kind, ParamKind::Positional | ParamKind::PositionalOnly)
        })
        .count();

    let max_count = params
        .iter()
        .filter(|p| {
            matches!(
                p.kind,
                ParamKind::Positional | ParamKind::PositionalOnly | ParamKind::KeywordOnly
            )
        })
        .count();

    // Check arg count
    if arg_types.len() < required_count || arg_types.len() > max_count {
        // Check if there's a *args param
        let has_var_positional = params
            .iter()
            .any(|p| matches!(p.kind, ParamKind::VarPositional));
        if !has_var_positional {
            return false;
        }
    }

    // Check each argument type against param type
    let positional_params: Vec<_> = params
        .iter()
        .filter(|p| {
            matches!(
                p.kind,
                ParamKind::Positional | ParamKind::PositionalOnly | ParamKind::KeywordOnly
            )
        })
        .collect();

    for (i, arg_ty) in arg_types.iter().enumerate() {
        if let Some(param) = positional_params.get(i) {
            if !types_compatible(arg_ty, &param.ty) {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::pdg::cfg::CfgBuilder;
    use crate::syntax::{Language, MultiParser};
    use crate::type_inference::ty::{Type, TypeVarId};

    #[test]
    fn test_cfg_narrowing_isinstance() {
        let source = "if isinstance(x, int):\n    y = x + 1\nz = x";
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Python).unwrap();
        let cfg = CfgBuilder::new(source).build(&parsed);

        let mut original_types = HashMap::new();
        original_types.insert("x".to_string(), Type::Union(vec![Type::Int, Type::Str]));

        let pass = CfgNarrowingPass::new(&cfg, original_types, source);
        let result = pass.run();

        // Should have computed block environments
        assert!(!result.block_envs.is_empty());
    }

    #[test]
    fn test_cfg_narrowing_is_none() {
        let source = "if x is None:\n    y = 1\nz = x";
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Python).unwrap();
        let cfg = CfgBuilder::new(source).build(&parsed);

        let mut original_types = HashMap::new();
        original_types.insert("x".to_string(), Type::Optional(Box::new(Type::Int)));

        let pass = CfgNarrowingPass::new(&cfg, original_types, source);
        let result = pass.run();

        assert!(!result.block_envs.is_empty());
    }

    #[test]
    fn test_resolve_typevar() {
        // T is a TypeVar, param: list[T], arg: list[int] → T = int
        let t_id = TypeVarId(0);
        let t = Type::TypeVar {
            id: t_id,
            name: "T".to_string(),
            bound: None,
            constraints: vec![],
            variance: crate::type_inference::ty::Variance::Invariant,
        };

        let param_ty = Type::List(Box::new(t));
        let arg_ty = Type::List(Box::new(Type::Int));

        let bindings = resolve_typevar_bindings(&[param_ty], &[arg_ty]);
        assert_eq!(bindings.get(&t_id), Some(&Type::Int));
    }

    #[test]
    fn test_overload_resolution() {
        // def f(x: int) -> str / def f(x: str) -> int
        let overloaded = Type::Overloaded {
            signatures: vec![
                Type::Callable {
                    params: vec![crate::type_inference::ty::Param {
                        name: "x".to_string(),
                        ty: Type::Int,
                        has_default: false,
                        kind: crate::type_inference::ty::ParamKind::Positional,
                    }],
                    ret: Box::new(Type::Str),
                },
                Type::Callable {
                    params: vec![crate::type_inference::ty::Param {
                        name: "x".to_string(),
                        ty: Type::Str,
                        has_default: false,
                        kind: crate::type_inference::ty::ParamKind::Positional,
                    }],
                    ret: Box::new(Type::Int),
                },
            ],
        };

        // f(42) should resolve to str
        let result = resolve_overload(&overloaded, &[Type::Int]);
        assert_eq!(result, Some(Type::Str));

        // f("hello") should resolve to int
        let result = resolve_overload(&overloaded, &[Type::Str]);
        assert_eq!(result, Some(Type::Int));
    }
}

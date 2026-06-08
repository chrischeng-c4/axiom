// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/gen/rust/logic.md#source
// CODEGEN-BEGIN
//! Logic/flowchart behavioral generator.
//!
//! Reads `LogicContent` parsed from Mermaid Plus frontmatter and generates:
//! - A function skeleton for the entry point
//! - Control flow scaffolds based on decision/process/terminal nodes
//! - SPEC-REF markers for each branch body (~20% coverage)
//!
//! All output lives inside CODEGEN-BEGIN/END markers.

// @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R3

use crate::generate::diagrams::content::logic::{FlowNode, FlowNodeKind, LogicContent};
use crate::generate::generators::primitive_registry::{lookup_by_name, substitute_template};
use crate::generate::marker::{emit_spec_ref, Lang};
use crate::generate::types::RustConfig;
use std::collections::{HashMap, HashSet};

/// Pre-computed cycle / loop information for a logic flowchart.
///
/// `loop_heads` are node ids that have at least one back-edge pointing to
/// them (computed via DFS over `LogicContent.entry`'s reachable subgraph).
/// `node_to_loop` maps each node that is a member of some SCC to the head
/// of that SCC; nodes outside any cycle are absent from the map.
///
/// /// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/logic-cycle-detection.md#logic (cycle detection flowchart)
#[derive(Debug, Clone, Default)]
struct LoopInfo {
    loop_heads: HashSet<String>,
    node_to_loop: HashMap<String, String>,
}

/// DFS over `content.entry` collecting nodes that are targets of back-edges
/// (i.e. cycle-entry / loop-head nodes).
fn find_loop_heads(content: &LogicContent) -> HashSet<String> {
    let mut heads = HashSet::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut stack: HashSet<String> = HashSet::new();
    fn dfs(
        node: &str,
        content: &LogicContent,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
        heads: &mut HashSet<String>,
    ) {
        if !visited.insert(node.to_string()) {
            return;
        }
        stack.insert(node.to_string());
        for edge in content.edges_from(node) {
            if stack.contains(&edge.to) {
                heads.insert(edge.to.clone());
            } else {
                dfs(&edge.to, content, visited, stack, heads);
            }
        }
        stack.remove(node);
    }
    dfs(
        &content.entry,
        content,
        &mut visited,
        &mut stack,
        &mut heads,
    );
    heads
}

/// Forward reachability: nodes reachable from `start` via outgoing edges.
fn reachable_forward(content: &LogicContent, start: &str) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut stack = vec![start.to_string()];
    while let Some(node) = stack.pop() {
        if !visited.insert(node.clone()) {
            continue;
        }
        for edge in content.edges_from(&node) {
            if !visited.contains(&edge.to) {
                stack.push(edge.to.clone());
            }
        }
    }
    visited
}

/// Backward reachability: nodes that can reach `target` via outgoing edges.
fn reachable_backward(content: &LogicContent, target: &str) -> HashSet<String> {
    let mut reverse: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &content.edges {
        reverse
            .entry(edge.to.clone())
            .or_default()
            .push(edge.from.clone());
    }
    let mut visited = HashSet::new();
    let mut stack = vec![target.to_string()];
    while let Some(node) = stack.pop() {
        if !visited.insert(node.clone()) {
            continue;
        }
        if let Some(parents) = reverse.get(&node) {
            for p in parents {
                if !visited.contains(p) {
                    stack.push(p.clone());
                }
            }
        }
    }
    visited
}

/// Compute SCC of a loop head: nodes both reachable from it and able to
/// reach it (i.e. participate in the cycle through `head`).
fn scc_members(content: &LogicContent, head: &str) -> HashSet<String> {
    let forward = reachable_forward(content, head);
    let backward = reachable_backward(content, head);
    forward.intersection(&backward).cloned().collect()
}

/// Build the full LoopInfo: heads + node→loop membership map.
fn detect_loops(content: &LogicContent) -> LoopInfo {
    let loop_heads = find_loop_heads(content);
    let mut node_to_loop: HashMap<String, String> = HashMap::new();
    for head in &loop_heads {
        for member in scc_members(content, head) {
            // Outermost-loop wins on ambiguity; deterministic fallback to first head encountered.
            node_to_loop.entry(member).or_insert_with(|| head.clone());
        }
    }
    LoopInfo {
        loop_heads,
        node_to_loop,
    }
}

/// Format the loop opening line for a loop head.
///
/// When `loop_kind == "for_each"` AND both `iter` and `loop_var` are
/// non-empty: emit `for <loop_var> in <iter> {`.
/// Otherwise (or on R11 malformed-input fallback): emit a comment line then
/// `loop {`. Returns the lines to push (1 or 2 elements).
///
/// /// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/logic-cycle-detection.md#logic (loop opening rules)
fn emit_loop_opening(node: &FlowNode, pad: &str) -> Vec<String> {
    if node.loop_kind.as_deref() == Some("for_each") {
        let iter = node.iter.as_deref().unwrap_or("");
        let loop_var = node.loop_var.as_deref().unwrap_or("");
        if !iter.is_empty() && !loop_var.is_empty() {
            return vec![format!("{}for {} in {} {{", pad, loop_var, iter)];
        }
        // R11: malformed for_each → fall back to generic loop with a comment.
        return vec![
            format!(
                "{}// (loop_kind=for_each but iter/loop_var missing — falling back to generic loop)",
                pad
            ),
            format!("{}loop {{", pad),
        ];
    }
    vec![format!("{}loop {{", pad)]
}

/// If `node` carries a `primitive: <name>` binding, look up the registry entry,
/// substitute `{out}`, `{T}`, and `{input_var}` placeholders, and return the
/// emitted code (one or more lines). Returns `None` if no primitive is bound
/// or the name isn't registered.
///
/// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#logic (gap-blocker #flowchart-to-fn)
fn emit_primitive_for_node(node: &FlowNode) -> Option<String> {
    let prim_name = node.primitive.as_deref()?;
    let entry = lookup_by_name(prim_name)?;

    let mut bindings: Vec<(&str, &str)> = Vec::new();
    if let Some(out) = node.output.as_deref() {
        bindings.push(("out", out));
    }
    if let Some(t) = node.type_param.as_deref() {
        bindings.push(("T", t));
    }
    for (k, v) in &node.inputs {
        bindings.push((k.as_str(), v.as_str()));
    }
    Some(substitute_template(entry.emit_template, &bindings))
}

/// Output from logic/flowchart code generation.
#[derive(Debug, Clone)]
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/logic.md#source
pub struct LogicGenOutput {
    /// The generated Rust function skeleton.
    pub code: String,
    /// SPEC-REF entries emitted inside the generated code.
    pub spec_refs: Vec<String>,
}

/// Generate a Rust function skeleton from a `LogicContent`.
///
/// The entry node becomes the function name. Decision nodes generate
/// placeholder `if` branches. Each branch body has a SPEC-REF marker.
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R3
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R4
pub fn generate_logic(
    content: &LogicContent,
    spec_path: &str,
    config: &RustConfig,
) -> LogicGenOutput {
    let vis = config.vis_prefix();
    let fn_name = content.entry.replace('-', "_");

    let mut spec_refs = Vec::new();
    let mut lines = Vec::new();

    // Function signature. Use a fully-qualified Result so the emitted code
    // compiles regardless of what the host file has imported; `anyhow::Result`
    // and a plain `Result<()>` both assume a context we can't guarantee at
    // codegen time. `todo!()` coerces to the error half so `return Ok(())`
    // from terminal nodes still type-checks.
    lines.push(format!(
        "{}fn {}() -> std::result::Result<(), Box<dyn std::error::Error>> {{",
        vis, fn_name,
    ));

    // Walk nodes in topological order (BFS from entry)
    let visited = generate_body(content, spec_path, &mut spec_refs, &mut lines, 1);

    // If entry node was not a decision, emit a plain SPEC-REF for the whole body
    if visited == 0 {
        let section_id = format!("{}-body", content.id);
        let marker = emit_spec_ref(
            spec_path,
            &section_id,
            &format!("Implement logic for {}", fn_name),
            Lang::Rust,
        );
        for marker_line in marker.lines() {
            lines.push(format!("    {}", marker_line));
        }
        spec_refs.push(format!("{}#{}", spec_path, section_id));
        lines.push("    todo!()".to_string());
    }

    lines.push("}".to_string());

    LogicGenOutput {
        code: lines.join("\n"),
        spec_refs,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::diagrams::content::logic::{FlowEdge, FlowNode, FlowNodeKind};
    use std::collections::HashMap;

    fn make_logic() -> LogicContent {
        let mut nodes = HashMap::new();
        nodes.insert(
            "start".to_string(),
            FlowNode {
                kind: FlowNodeKind::Start,
                label: Some("Begin".to_string()),
                ..Default::default()
            },
        );
        nodes.insert(
            "validate_input".to_string(),
            FlowNode {
                kind: FlowNodeKind::Decision,
                label: Some("Valid?".to_string()),
                fn_name: Some("is_valid".to_string()),
                ..Default::default()
            },
        );
        nodes.insert(
            "return_error".to_string(),
            FlowNode {
                kind: FlowNodeKind::Terminal,
                label: Some("Return error".to_string()),
                ..Default::default()
            },
        );
        nodes.insert(
            "process_ok".to_string(),
            FlowNode {
                kind: FlowNodeKind::Terminal,
                label: Some("Return ok".to_string()),
                ..Default::default()
            },
        );

        LogicContent {
            id: "validate".to_string(),
            entry: "start".to_string(),
            nodes,
            edges: vec![
                FlowEdge {
                    from: "start".to_string(),
                    to: "validate_input".to_string(),
                    label: None,
                },
                FlowEdge {
                    from: "validate_input".to_string(),
                    to: "return_error".to_string(),
                    label: Some("no".to_string()),
                },
                FlowEdge {
                    from: "validate_input".to_string(),
                    to: "process_ok".to_string(),
                    label: Some("yes".to_string()),
                },
            ],
            title: None,
        }
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R3
    #[test]
    fn test_generates_function_skeleton() {
        let logic = make_logic();
        let config = crate::generate::types::RustConfig::default();
        let output = generate_logic(&logic, "spec.md", &config);

        // Function signature based on entry node
        assert!(
            output.code.contains("fn start"),
            "Should have fn with entry name"
        );
        assert!(
            output.code.contains("std::result::Result<()"),
            "Should return fully-qualified Result, got:\n{}",
            output.code,
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R3
    #[test]
    fn test_generates_decision_branches() {
        let logic = make_logic();
        let config = crate::generate::types::RustConfig::default();
        let output = generate_logic(&logic, "spec.md", &config);

        // "Valid?" — question-form label, translated to is_valid()
        assert!(
            output.code.contains("Decision:"),
            "Should have Decision comment"
        );
        assert!(
            output.code.contains("if is_valid()"),
            "Should translate 'Valid?' into is_valid() call, got:\n{}",
            output.code
        );
        // Branch labels kept as inline comments on the if/else
        assert!(
            output.code.contains("/* yes */") || output.code.contains("/* no */"),
            "Should keep branch labels as comments"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R4
    #[test]
    fn test_generates_spec_ref_for_process_branches() {
        // Decision-only graphs with terminal leaves emit real return
        // statements — no SPEC-REF fallback needed. SPEC-REFs only fire
        // for Process steps (where a human must still implement the call)
        // or for completely empty branches.
        let mut nodes = HashMap::new();
        nodes.insert(
            "start".to_string(),
            FlowNode {
                kind: FlowNodeKind::Start,
                ..Default::default()
            },
        );
        nodes.insert(
            "parse".to_string(),
            FlowNode {
                kind: FlowNodeKind::Process,
                label: Some("Parse input".to_string()),
                ..Default::default()
            },
        );
        nodes.insert(
            "done".to_string(),
            FlowNode {
                kind: FlowNodeKind::Terminal,
                label: Some("ok".to_string()),
                ..Default::default()
            },
        );
        let content = LogicContent {
            id: "ref_test".to_string(),
            entry: "start".to_string(),
            nodes,
            edges: vec![
                FlowEdge {
                    from: "start".into(),
                    to: "parse".into(),
                    label: None,
                },
                FlowEdge {
                    from: "parse".into(),
                    to: "done".into(),
                    label: None,
                },
            ],
            title: None,
        };
        let config = crate::generate::types::RustConfig::default();
        let output = generate_logic(&content, "spec.md", &config);

        assert!(
            output.code.contains("SPEC-REF"),
            "Process node should emit SPEC-REF marker"
        );
        assert!(
            !output.spec_refs.is_empty(),
            "Should emit spec_refs for process nodes"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R3
    #[test]
    fn test_generates_terminal_node_comments() {
        let logic = make_logic();
        let config = crate::generate::types::RustConfig::default();
        let output = generate_logic(&logic, "spec.md", &config);

        // Terminal nodes emit in-place return statements plus a trailing
        // catalogue comment.
        assert!(
            output.code.contains("Terminal:"),
            "Should keep terminal catalogue comments"
        );
        assert!(
            output.code.contains("return Err(todo!"),
            "Error terminal should emit return Err, got:\n{}",
            output.code
        );
        assert!(
            output.code.contains("return Ok(())"),
            "Ok terminal should emit return Ok(()), got:\n{}",
            output.code
        );
    }

    #[test]
    fn test_label_to_condition_fn_name_wins() {
        // When fn_name is declared, emit a real call to that identifier.
        assert_eq!(
            label_to_condition(Some("is_valid"), Some("Valid?"), "node"),
            "is_valid()",
        );
        assert_eq!(
            label_to_condition(Some("has_permission"), Some("Has Permission?"), "n"),
            "has_permission()",
        );
    }

    #[test]
    fn test_label_to_condition_comparison_passthrough() {
        // No fn_name, but label is a comparison → pass through verbatim.
        assert_eq!(
            label_to_condition(None, Some("count > 0"), "n"),
            "count > 0",
            "Comparisons pass through verbatim",
        );
        assert_eq!(label_to_condition(None, Some("x == y"), "n"), "x == y");
    }

    #[test]
    fn test_label_to_condition_todo_default() {
        // No fn_name + non-comparison label → todo!("decision: ...") so
        // the generated file compiles without referencing undefined helpers.
        assert_eq!(
            label_to_condition(None, None, "my_node"),
            "todo!(\"decision: my_node\")",
        );
        assert_eq!(
            label_to_condition(None, Some(""), "my_node"),
            "todo!(\"decision: my_node\")",
        );
        assert_eq!(
            label_to_condition(None, Some("Valid?"), "n"),
            "todo!(\"decision: Valid?\")",
        );
        assert_eq!(
            label_to_condition(None, Some("check input"), "n"),
            "todo!(\"decision: check input\")",
        );
        assert_eq!(
            label_to_condition(None, Some("rendered width at indent=0 <= 100?"), "n"),
            "todo!(\"decision: rendered width at indent=0 <= 100?\")",
        );
    }

    #[test]
    fn test_label_to_terminal_ok_family() {
        assert_eq!(label_to_terminal(Some("ok"), "n"), "return Ok(())");
        assert_eq!(label_to_terminal(Some("Return ok"), "n"), "return Ok(())");
        assert_eq!(label_to_terminal(Some("success"), "n"), "return Ok(())");
        assert_eq!(label_to_terminal(Some("DONE"), "n"), "return Ok(())");
    }

    #[test]
    fn test_label_to_terminal_error_family() {
        assert!(label_to_terminal(Some("error"), "n").starts_with("return Err(todo!"));
        assert!(label_to_terminal(Some("Return error"), "n").starts_with("return Err(todo!"));
        assert!(label_to_terminal(Some("fail validation"), "n").starts_with("return Err(todo!"));
        assert!(label_to_terminal(Some("rejected"), "n").starts_with("return Err(todo!"));
    }

    #[test]
    fn test_label_to_terminal_unknown_falls_back_to_todo() {
        let out = label_to_terminal(Some("log metric"), "node");
        assert!(
            out.starts_with("todo!("),
            "Unknown terminal label should hit generic todo!, got: {}",
            out
        );
        assert!(out.contains("log metric"));
    }

    #[test]
    fn test_snake_case_identifier_safe() {
        assert_eq!(snake_case("Valid"), "valid");
        assert_eq!(snake_case("Has Permission"), "has_permission");
        assert_eq!(snake_case("foo-bar_baz"), "foo_bar_baz");
        assert_eq!(snake_case("HTTP 200 OK"), "http_200_ok");
        assert_eq!(snake_case("---"), "_");
        assert_eq!(snake_case("2legit"), "_2legit");
    }

    #[test]
    fn test_label_to_process_with_fn_name_and_params() {
        let params = vec!["input".to_string(), "config".to_string()];
        assert_eq!(
            label_to_process(Some("parse_input"), Some("Parse input"), "n", &params, None),
            "parse_input(input, config)",
        );
    }

    #[test]
    fn test_label_to_process_async_appends_await() {
        let out = label_to_process(Some("fetch_data"), Some("Fetch data"), "n", &[], Some(true));
        assert_eq!(out, "fetch_data().await");

        // With params + async.
        let params = vec!["url".to_string()];
        let out = label_to_process(Some("fetch"), Some("Fetch"), "n", &params, Some(true));
        assert_eq!(out, "fetch(url).await");
    }

    #[test]
    fn test_label_to_process_async_false_stays_sync() {
        let out = label_to_process(Some("foo"), Some("Foo"), "n", &[], Some(false));
        assert_eq!(out, "foo()");
        let out = label_to_process(Some("foo"), Some("Foo"), "n", &[], None);
        assert_eq!(out, "foo()");
    }

    #[test]
    fn test_label_to_process_todo_default() {
        // No fn_name → todo! with the label so the generated file compiles.
        let out = label_to_process(None, Some("Parse input"), "n", &[], None);
        assert_eq!(out, "todo!(\"process: Parse input\")");

        // No fn_name, no label → fall back to node id.
        let out = label_to_process(None, None, "my_node", &[], None);
        assert_eq!(out, "todo!(\"process: my_node\")");

        // params and is_async are ignored without fn_name — there is no
        // call to attach them to.
        let params = vec!["a".to_string(), "b".to_string()];
        let out = label_to_process(None, Some("Run"), "n", &params, Some(true));
        assert_eq!(out, "todo!(\"process: Run\")");
    }

    #[test]
    fn test_process_node_emits_method_call() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "start".to_string(),
            FlowNode {
                kind: FlowNodeKind::Start,
                ..Default::default()
            },
        );
        nodes.insert(
            "parse".to_string(),
            FlowNode {
                kind: FlowNodeKind::Process,
                label: Some("Parse input".to_string()),
                fn_name: Some("parse_input".to_string()),
                ..Default::default()
            },
        );
        nodes.insert(
            "done".to_string(),
            FlowNode {
                kind: FlowNodeKind::Terminal,
                label: Some("ok".to_string()),
                ..Default::default()
            },
        );
        let content = LogicContent {
            id: "proc_test".to_string(),
            entry: "start".to_string(),
            nodes,
            edges: vec![
                FlowEdge {
                    from: "start".into(),
                    to: "parse".into(),
                    label: None,
                },
                FlowEdge {
                    from: "parse".into(),
                    to: "done".into(),
                    label: None,
                },
            ],
            title: None,
        };
        let config = crate::generate::types::RustConfig::default();
        let out = generate_logic(&content, "spec.md", &config);
        assert!(
            out.code.contains("parse_input();"),
            "Process node should emit label-derived method call, got:\n{}",
            out.code
        );
        assert!(
            out.code.contains("return Ok(())"),
            "Terminal should emit inline return, got:\n{}",
            out.code
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Cycle handling tests (R10/R11 from
    // enhancement-cycle-detection-loop-emission-in-generate-logic-cl)
    // ─────────────────────────────────────────────────────────────────────

    /// Self-loop (single-node cycle): node A has edge A→A and edge A→exit.
    /// Should emit `loop { ... }` with `continue;` on the back-edge and
    /// `break;` on the cycle-exit edge, then recurse exit outside the loop.
    /// REQ projects/agentic-workflow/tech-design/core/generate/gen/rust/logic-cycle-detection.md — R5 (generic loop) + R7 (continue) + R8 (break)
    #[test]
    fn test_emit_node_self_loop_emits_loop_form() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "start".to_string(),
            FlowNode {
                kind: FlowNodeKind::Start,
                ..Default::default()
            },
        );
        nodes.insert(
            "loop_node".to_string(),
            FlowNode {
                kind: FlowNodeKind::Decision,
                label: Some("more?".to_string()),
                ..Default::default()
            },
        );
        nodes.insert(
            "exit".to_string(),
            FlowNode {
                kind: FlowNodeKind::Terminal,
                label: Some("ok".to_string()),
                ..Default::default()
            },
        );
        let content = LogicContent {
            id: "self_loop".to_string(),
            entry: "start".to_string(),
            nodes,
            edges: vec![
                FlowEdge {
                    from: "start".into(),
                    to: "loop_node".into(),
                    label: None,
                },
                FlowEdge {
                    from: "loop_node".into(),
                    to: "loop_node".into(),
                    label: Some("yes".into()),
                },
                FlowEdge {
                    from: "loop_node".into(),
                    to: "exit".into(),
                    label: Some("no".into()),
                },
            ],
            title: None,
        };
        let config = crate::generate::types::RustConfig::default();
        let out = generate_logic(&content, "spec.md", &config);
        assert!(
            out.code.contains("loop {"),
            "must open generic loop, got:\n{}",
            out.code
        );
        assert!(
            out.code.contains("continue;"),
            "back-edge to self must emit continue, got:\n{}",
            out.code
        );
        assert!(
            out.code.contains("break;"),
            "cycle-exit must emit break, got:\n{}",
            out.code
        );
        assert!(
            out.code.contains("return Ok(())"),
            "exit terminal must still emit return after loop closes, got:\n{}",
            out.code
        );
        // Crucially: must NOT bail at MAX_DEPTH
        assert!(
            !out.code.contains("(max nesting depth reached)"),
            "cycle handling must not fall through to MAX_DEPTH guard, got:\n{}",
            out.code
        );
    }

    /// Two-node cycle: A is loop head, A→B→A is the cycle, B→C is the exit.
    /// REQ projects/agentic-workflow/tech-design/core/generate/gen/rust/logic-cycle-detection.md — R5 (generic loop) on multi-node cycle
    #[test]
    fn test_emit_node_two_node_cycle_emits_loop_form() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "start".to_string(),
            FlowNode {
                kind: FlowNodeKind::Start,
                ..Default::default()
            },
        );
        nodes.insert(
            "a".to_string(),
            FlowNode {
                kind: FlowNodeKind::Process,
                label: Some("Step A".to_string()),
                fn_name: Some("step_a".to_string()),
                ..Default::default()
            },
        );
        nodes.insert(
            "b".to_string(),
            FlowNode {
                kind: FlowNodeKind::Decision,
                label: Some("done?".to_string()),
                ..Default::default()
            },
        );
        nodes.insert(
            "c".to_string(),
            FlowNode {
                kind: FlowNodeKind::Terminal,
                label: Some("ok".to_string()),
                ..Default::default()
            },
        );
        let content = LogicContent {
            id: "two_node_cycle".to_string(),
            entry: "start".to_string(),
            nodes,
            edges: vec![
                FlowEdge {
                    from: "start".into(),
                    to: "a".into(),
                    label: None,
                },
                FlowEdge {
                    from: "a".into(),
                    to: "b".into(),
                    label: None,
                },
                FlowEdge {
                    from: "b".into(),
                    to: "a".into(),
                    label: Some("no".into()),
                },
                FlowEdge {
                    from: "b".into(),
                    to: "c".into(),
                    label: Some("yes".into()),
                },
            ],
            title: None,
        };
        let config = crate::generate::types::RustConfig::default();
        let out = generate_logic(&content, "spec.md", &config);
        assert!(
            out.code.contains("loop {"),
            "must open loop, got:\n{}",
            out.code
        );
        assert!(
            out.code.contains("step_a();"),
            "process body must emit inside loop, got:\n{}",
            out.code
        );
        assert!(
            out.code.contains("continue;"),
            "B→A back-edge must emit continue, got:\n{}",
            out.code
        );
        assert!(
            out.code.contains("break;"),
            "B→C exit edge must emit break, got:\n{}",
            out.code
        );
        assert!(
            out.code.contains("return Ok(())"),
            "exit terminal must emit return outside the loop, got:\n{}",
            out.code
        );
        assert!(
            !out.code.contains("(max nesting depth reached)"),
            "cycle handling must not bail at MAX_DEPTH, got:\n{}",
            out.code
        );
    }

    /// `loop_kind: for_each` with non-empty `iter` and `loop_var` emits
    /// `for <loop_var> in <iter> { ... }` instead of generic `loop { ... }`.
    /// REQ projects/agentic-workflow/tech-design/core/generate/gen/rust/logic-cycle-detection.md — R6 (for_each form)
    #[test]
    fn test_emit_node_for_each_cycle_emits_for_loop() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "start".to_string(),
            FlowNode {
                kind: FlowNodeKind::Start,
                ..Default::default()
            },
        );
        nodes.insert(
            "loop_head".to_string(),
            FlowNode {
                kind: FlowNodeKind::Decision,
                label: Some("more?".to_string()),
                loop_kind: Some("for_each".to_string()),
                iter: Some("reader.lines()".to_string()),
                loop_var: Some("line".to_string()),
                ..Default::default()
            },
        );
        nodes.insert(
            "exit".to_string(),
            FlowNode {
                kind: FlowNodeKind::Terminal,
                label: Some("ok".to_string()),
                ..Default::default()
            },
        );
        let content = LogicContent {
            id: "for_each_cycle".to_string(),
            entry: "start".to_string(),
            nodes,
            edges: vec![
                FlowEdge {
                    from: "start".into(),
                    to: "loop_head".into(),
                    label: None,
                },
                FlowEdge {
                    from: "loop_head".into(),
                    to: "loop_head".into(),
                    label: Some("yes".into()),
                },
                FlowEdge {
                    from: "loop_head".into(),
                    to: "exit".into(),
                    label: Some("no".into()),
                },
            ],
            title: None,
        };
        let config = crate::generate::types::RustConfig::default();
        let out = generate_logic(&content, "spec.md", &config);
        assert!(
            out.code.contains("for line in reader.lines() {"),
            "for_each must emit `for <loop_var> in <iter> {{` form, got:\n{}",
            out.code,
        );
        assert!(
            !out.code.contains("loop {"),
            "for_each must NOT also emit generic `loop {{`, got:\n{}",
            out.code
        );
        assert!(
            out.code.contains("break;"),
            "cycle-exit must still emit break, got:\n{}",
            out.code
        );
    }

    /// Non-cyclic graph emits identical Rust output before/after this change.
    /// REQ projects/agentic-workflow/tech-design/core/generate/gen/rust/logic-cycle-detection.md — R9 (regression guard)
    #[test]
    fn test_emit_node_non_cyclic_unchanged() {
        // Build a simple linear graph: start → process → terminal.
        // No cycles. The output should not contain loop/continue/break.
        let mut nodes = HashMap::new();
        nodes.insert(
            "start".to_string(),
            FlowNode {
                kind: FlowNodeKind::Start,
                ..Default::default()
            },
        );
        nodes.insert(
            "p".to_string(),
            FlowNode {
                kind: FlowNodeKind::Process,
                label: Some("Do work".to_string()),
                fn_name: Some("do_work".to_string()),
                ..Default::default()
            },
        );
        nodes.insert(
            "done".to_string(),
            FlowNode {
                kind: FlowNodeKind::Terminal,
                label: Some("ok".to_string()),
                ..Default::default()
            },
        );
        let content = LogicContent {
            id: "linear".to_string(),
            entry: "start".to_string(),
            nodes,
            edges: vec![
                FlowEdge {
                    from: "start".into(),
                    to: "p".into(),
                    label: None,
                },
                FlowEdge {
                    from: "p".into(),
                    to: "done".into(),
                    label: None,
                },
            ],
            title: None,
        };
        let config = crate::generate::types::RustConfig::default();
        let out = generate_logic(&content, "spec.md", &config);
        assert!(
            out.code.contains("do_work();"),
            "process emits, got:\n{}",
            out.code
        );
        assert!(
            out.code.contains("return Ok(())"),
            "terminal emits, got:\n{}",
            out.code
        );
        assert!(
            !out.code.contains("loop {"),
            "non-cyclic must NOT emit `loop {{`, got:\n{}",
            out.code
        );
        assert!(
            !out.code.contains("continue;"),
            "non-cyclic must NOT emit continue, got:\n{}",
            out.code
        );
        assert!(
            !out.code.contains("break;"),
            "non-cyclic must NOT emit break, got:\n{}",
            out.code
        );
    }

    /// `loop_kind: for_each` with absent `iter` falls back to generic `loop`
    /// + a documenting comment per R11. No panic, no `compile_error!()`.
    /// REQ projects/agentic-workflow/tech-design/core/generate/gen/rust/logic-cycle-detection.md — R11 (malformed for_each fallback)
    #[test]
    fn test_emit_node_for_each_malformed_falls_back_to_generic_loop() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "start".to_string(),
            FlowNode {
                kind: FlowNodeKind::Start,
                ..Default::default()
            },
        );
        nodes.insert(
            "loop_head".to_string(),
            FlowNode {
                kind: FlowNodeKind::Decision,
                label: Some("more?".to_string()),
                // for_each set, but iter and loop_var both None — malformed
                loop_kind: Some("for_each".to_string()),
                iter: None,
                loop_var: None,
                ..Default::default()
            },
        );
        nodes.insert(
            "exit".to_string(),
            FlowNode {
                kind: FlowNodeKind::Terminal,
                label: Some("ok".to_string()),
                ..Default::default()
            },
        );
        let content = LogicContent {
            id: "malformed_for_each".to_string(),
            entry: "start".to_string(),
            nodes,
            edges: vec![
                FlowEdge {
                    from: "start".into(),
                    to: "loop_head".into(),
                    label: None,
                },
                FlowEdge {
                    from: "loop_head".into(),
                    to: "loop_head".into(),
                    label: Some("yes".into()),
                },
                FlowEdge {
                    from: "loop_head".into(),
                    to: "exit".into(),
                    label: Some("no".into()),
                },
            ],
            title: None,
        };
        let config = crate::generate::types::RustConfig::default();
        let out = generate_logic(&content, "spec.md", &config);
        assert!(
            out.code.contains(
                "// (loop_kind=for_each but iter/loop_var missing — falling back to generic loop)"
            ),
            "must emit fallback comment per R11, got:\n{}",
            out.code,
        );
        assert!(
            out.code.contains("loop {"),
            "must fall back to generic `loop {{`, got:\n{}",
            out.code
        );
        assert!(
            !out.code.contains("compile_error!"),
            "must NOT emit compile_error! per R11, got:\n{}",
            out.code
        );
        assert!(
            !out.code.contains("for  in "),
            "must NOT emit malformed `for  in `, got:\n{}",
            out.code
        );
    }

    /// Process node with `primitive: parse_jsonl_stream` should emit the
    /// registry's substituted emit_template instead of a method call.
    /// REQ projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#logic — gap-blocker #flowchart-to-fn
    #[test]
    fn test_process_node_with_primitive_emits_template() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "start".to_string(),
            FlowNode {
                kind: FlowNodeKind::Start,
                ..Default::default()
            },
        );
        let mut inputs = HashMap::new();
        inputs.insert("path".to_string(), "CHANNEL_PATH".to_string());
        nodes.insert(
            "read_msgs".to_string(),
            FlowNode {
                kind: FlowNodeKind::Process,
                label: Some("Parse channel".to_string()),
                primitive: Some("parse_jsonl_stream".to_string()),
                inputs,
                output: Some("msgs".to_string()),
                type_param: Some("ChannelMessage".to_string()),
                ..Default::default()
            },
        );
        nodes.insert(
            "done".to_string(),
            FlowNode {
                kind: FlowNodeKind::Terminal,
                label: Some("ok".to_string()),
                ..Default::default()
            },
        );
        let content = LogicContent {
            id: "primitive_test".to_string(),
            entry: "start".to_string(),
            nodes,
            edges: vec![
                FlowEdge {
                    from: "start".into(),
                    to: "read_msgs".into(),
                    label: None,
                },
                FlowEdge {
                    from: "read_msgs".into(),
                    to: "done".into(),
                    label: None,
                },
            ],
            title: None,
        };
        let config = crate::generate::types::RustConfig::default();
        let out = generate_logic(&content, "spec.md", &config);
        assert!(
            out.code.contains("let msgs: Vec<ChannelMessage>"),
            "primitive should substitute {{out}} and {{T}}, got:\n{}",
            out.code,
        );
        assert!(
            out.code.contains("CHANNEL_PATH"),
            "primitive should substitute {{path}} input binding, got:\n{}",
            out.code,
        );
        assert!(
            out.code.contains("BufReader") || out.code.contains("serde_json::from_str"),
            "primitive should emit JSONL parse body, got:\n{}",
            out.code,
        );
        // Crucially: must NOT fall back to label_to_process todo!()
        assert!(
            !out.code.contains("todo!(\"process: Parse channel\")"),
            "primitive node must not emit todo! placeholder, got:\n{}",
            out.code,
        );
    }

    /// Process node with `primitive: append_line_atomic` substitutes the
    /// O_APPEND emit template via `{path}` and `{value}`.
    /// REQ projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#logic — gap-blocker #flowchart-to-fn
    #[test]
    fn test_process_node_primitive_append_line_atomic() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "start".to_string(),
            FlowNode {
                kind: FlowNodeKind::Start,
                ..Default::default()
            },
        );
        let mut inputs = HashMap::new();
        inputs.insert("path".to_string(), "CHANNEL_PATH".to_string());
        inputs.insert("value".to_string(), "msg".to_string());
        nodes.insert(
            "write_msg".to_string(),
            FlowNode {
                kind: FlowNodeKind::Process,
                label: Some("Write JSONL line".to_string()),
                primitive: Some("append_line_atomic".to_string()),
                inputs,
                ..Default::default()
            },
        );
        nodes.insert(
            "done".to_string(),
            FlowNode {
                kind: FlowNodeKind::Terminal,
                label: Some("ok".to_string()),
                ..Default::default()
            },
        );
        let content = LogicContent {
            id: "atomic_append_test".to_string(),
            entry: "start".to_string(),
            nodes,
            edges: vec![
                FlowEdge {
                    from: "start".into(),
                    to: "write_msg".into(),
                    label: None,
                },
                FlowEdge {
                    from: "write_msg".into(),
                    to: "done".into(),
                    label: None,
                },
            ],
            title: None,
        };
        let config = crate::generate::types::RustConfig::default();
        let out = generate_logic(&content, "spec.md", &config);
        assert!(
            out.code.contains("OpenOptions") && out.code.contains("append(true)"),
            "primitive must emit O_APPEND open, got:\n{}",
            out.code,
        );
        assert!(
            out.code.contains("serde_json::to_string(&msg)"),
            "primitive must substitute {{value}} into to_string call, got:\n{}",
            out.code,
        );
        assert!(
            out.code.contains("CHANNEL_PATH"),
            "primitive must substitute {{path}} into open call, got:\n{}",
            out.code,
        );
    }

    /// Unknown primitive name falls back to label_to_process (no panic).
    /// REQ projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#logic — defensive
    #[test]
    fn test_process_node_unknown_primitive_falls_back_to_todo() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "start".to_string(),
            FlowNode {
                kind: FlowNodeKind::Start,
                ..Default::default()
            },
        );
        nodes.insert(
            "p".to_string(),
            FlowNode {
                kind: FlowNodeKind::Process,
                label: Some("Unknown".to_string()),
                primitive: Some("primitive_does_not_exist".to_string()),
                ..Default::default()
            },
        );
        nodes.insert(
            "done".to_string(),
            FlowNode {
                kind: FlowNodeKind::Terminal,
                label: Some("ok".to_string()),
                ..Default::default()
            },
        );
        let content = LogicContent {
            id: "unknown_prim_test".to_string(),
            entry: "start".to_string(),
            nodes,
            edges: vec![
                FlowEdge {
                    from: "start".into(),
                    to: "p".into(),
                    label: None,
                },
                FlowEdge {
                    from: "p".into(),
                    to: "done".into(),
                    label: None,
                },
            ],
            title: None,
        };
        let config = crate::generate::types::RustConfig::default();
        let out = generate_logic(&content, "spec.md", &config);
        assert!(
            out.code.contains("todo!(\"process: Unknown\")"),
            "unknown primitive name should fall back to label_to_process todo!, got:\n{}",
            out.code,
        );
    }
}

/// Generate control flow body via graph traversal from entry node.
/// Returns number of decision branches emitted.
fn generate_body(
    content: &LogicContent,
    spec_path: &str,
    spec_refs: &mut Vec<String>,
    lines: &mut Vec<String>,
    indent: usize,
) -> usize {
    let loop_info = detect_loops(content);
    let mut visiting: HashSet<String> = HashSet::new();
    let mut continuations: Vec<String> = Vec::new();
    let branches = emit_node(
        content,
        &content.entry,
        spec_path,
        spec_refs,
        lines,
        indent,
        0,
        &mut visiting,
        &loop_info,
        &mut continuations,
    );

    // Emit a trailing `// terminals:` catalogue so authors can see at a
    // glance which terminal IDs + labels the graph defined. These are NOT
    // control flow — inline `return` statements (emitted when visiting the
    // Terminal kind directly) are the actual emission.
    let pad = "    ".repeat(indent);
    let mut terminal_ids = content.terminal_ids();
    terminal_ids.sort();
    for terminal_id in terminal_ids {
        let label = content.nodes[terminal_id]
            .label
            .as_deref()
            .unwrap_or(terminal_id);
        lines.push(format!("{}// Terminal: {} -> {}", pad, terminal_id, label));
    }

    branches
}

/// Convert a decision node into a Rust expression usable as an `if`
/// condition. Resolution order:
///
/// - Node declares `fn_name: <ident>` in Mermaid Plus → emit `<fn_name>()`.
///   Authors opt in to real calls this way (extension per
///   `.aw/tech-design/.../mermaid-plus.md`).
/// - Label contains `==` / `!=` / `<` / `>` / `<=` / `>=` — assume caller
///   wrote a real Rust comparison, pass through verbatim (trimmed).
/// - Everything else → `todo!("decision: <label-or-id>")`. Deriving call
///   names from free-text labels used to produce references to functions
///   that don't exist anywhere in the crate (E0425 by the hundreds);
///   `todo!()` has type `!` so it coerces to `bool`, the file compiles,
///   and the author can grep `todo!("decision:` to fill in real logic.
///
/// Consistent output means Rule 1 audit's regenerate-diff check will pass
/// as long as the spec's label doesn't change.
fn label_to_condition(fn_name: Option<&str>, label: Option<&str>, node_id: &str) -> String {
    if let Some(fn_ident) = fn_name.map(str::trim).filter(|s| !s.is_empty()) {
        return format!("{}()", fn_ident);
    }
    let label_txt = label.map(str::trim).filter(|s| !s.is_empty());
    // Comparison-shaped labels pass through verbatim.
    if let Some(raw) = label_txt {
        if contains_rust_comparison(raw) {
            return raw.to_string();
        }
    }
    let display = label_txt.unwrap_or(node_id);
    format!("todo!(\"decision: {}\")", escape_todo_msg(display))
}

/// Turn a terminal node's label into a `return <expr>` Rust statement.
/// Recognised patterns (case-insensitive):
///
/// - `ok`, `success`, `done`, or a label starting with `return ok` /
///   `return success` → `return Ok(())`.
/// - `error`, `fail`, `reject`, `denied` → `return Err(...)` with a
///   `todo!()` error-value placeholder that at least compiles.
/// - everything else → `todo!("terminal: <label>")`, keyed on the label so
///   authors can grep the TODOs.
fn label_to_terminal(label: Option<&str>, node_id: &str) -> String {
    let raw = label
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(node_id);
    let low = raw.to_ascii_lowercase();
    if low == "ok"
        || low == "success"
        || low == "done"
        || low.starts_with("return ok")
        || low.starts_with("return success")
    {
        "return Ok(())".to_string()
    } else if low.contains("error")
        || low.contains("fail")
        || low.contains("reject")
        || low.contains("denied")
    {
        format!("return Err(todo!(\"error: {}\"))", raw)
    } else {
        format!("todo!(\"terminal: {}\")", raw)
    }
}

/// Turn a process node into an action statement. Resolution order:
///
/// - Node declares `fn_name: <ident>` → emit `<fn_name>(<params>)` (optionally
///   `.await` when `is_async` is `Some(true)`). Authors opt in to real
///   calls via this field.
/// - Otherwise → `todo!("process: <label-or-id>")`. Previously the
///   generator snake_cased the label and emitted it as a function call,
///   producing references to undefined helpers; `todo!()` keeps the file
///   compilable and gives authors a greppable marker (`todo!("process:`).
fn label_to_process(
    fn_name: Option<&str>,
    label: Option<&str>,
    node_id: &str,
    params: &[String],
    is_async: Option<bool>,
) -> String {
    if let Some(fn_ident) = fn_name.map(str::trim).filter(|s| !s.is_empty()) {
        let args = params.join(", ");
        let call = format!("{}({})", fn_ident, args);
        return if matches!(is_async, Some(true)) {
            format!("{}.await", call)
        } else {
            call
        };
    }
    let display = label
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(node_id);
    format!("todo!(\"process: {}\")", escape_todo_msg(display))
}

/// Escape backslashes and double quotes inside a label so it can be inlined
/// into the `todo!("...")` string literal without breaking lexing.
fn escape_todo_msg(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push(' '),
            _ => out.push(c),
        }
    }
    out
}

/// Detect a Rust comparison operator in a free-form label. Strict match —
/// we want to pass through author-written Rust, not turn prose like
/// "Less than 10 items" into nonsense.
fn contains_rust_comparison(s: &str) -> bool {
    if s.ends_with('?') {
        return false;
    }
    let mut parts = s.split_whitespace();
    let Some(lhs) = parts.next() else {
        return false;
    };
    let Some(op) = parts.next() else {
        return false;
    };
    let Some(rhs) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }
    matches!(op, "==" | "!=" | "<=" | ">=" | "<" | ">")
        && is_rust_condition_atom(lhs)
        && is_rust_condition_atom(rhs)
}

fn is_rust_condition_atom(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '.' | '(' | ')' | '[' | ']')
        })
}

/// Convert any string into a reasonable Rust-identifier snake_case form.
/// Non-alphanumeric → `_`; collapses doubled underscores; lowercases.
/// Never emits a leading digit (prepends `_` if needed) or empty string
/// (returns `_`).
#[allow(dead_code)]
fn snake_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_us = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            for lc in c.to_lowercase() {
                out.push(lc);
            }
            prev_us = false;
        } else if !prev_us && !out.is_empty() {
            out.push('_');
            prev_us = true;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        "_".to_string()
    } else if out.chars().next().unwrap().is_ascii_digit() {
        format!("_{}", out)
    } else {
        out
    }
}

/// Recursive graph traversal emitting code for a single node.
///
/// Cycle handling (added by enhancement-cycle-detection-loop-emission-in-generate-logic-cl):
///   - `visiting` tracks the current DFS path; back-edges (target in `visiting`)
///     emit `continue;` instead of recursing.
///   - `loop_info` is precomputed loop-head + SCC membership.
///   - `continuations` is a sink for cycle-exit edges' targets, consumed by
///     the enclosing loop head after it closes its `}`.
///
/// /// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/logic-cycle-detection.md#logic (cycle detection flowchart)
#[allow(clippy::too_many_arguments)]
fn emit_node(
    content: &LogicContent,
    node_id: &str,
    spec_path: &str,
    spec_refs: &mut Vec<String>,
    lines: &mut Vec<String>,
    indent: usize,
    depth: usize,
    visiting: &mut HashSet<String>,
    loop_info: &LoopInfo,
    continuations: &mut Vec<String>,
) -> usize {
    const MAX_DEPTH: usize = 20;
    if depth > MAX_DEPTH {
        let pad = "    ".repeat(indent);
        lines.push(format!("{}// (max nesting depth reached)", pad));
        return 0;
    }

    let node = match content.nodes.get(node_id) {
        Some(n) => n,
        None => return 0,
    };

    visiting.insert(node_id.to_string());
    let pad = "    ".repeat(indent);
    let edges = content.edges_from(node_id);

    // ── Loop-head wrapping ──
    // If this node is a loop head (has incoming back-edges), wrap its body
    // in `loop { ... }` (or `for ... in ... { ... }` for `for_each` form).
    // Cycle-exit edges encountered inside the body bubble up via the local
    // `inner_continuations` Vec; they are emitted recursively AFTER the
    // closing `}` so the loop's break-into-continuation flow reads naturally.
    let is_loop_head = loop_info.loop_heads.contains(node_id);
    let inner_indent = if is_loop_head { indent + 1 } else { indent };
    let mut inner_continuations: Vec<String> = Vec::new();
    let active_continuations: &mut Vec<String> = if is_loop_head {
        &mut inner_continuations
    } else {
        continuations
    };

    if is_loop_head {
        for line in emit_loop_opening(node, &pad) {
            lines.push(line);
        }
    }

    let inner_pad = "    ".repeat(inner_indent);

    let branches = match node.kind {
        FlowNodeKind::Start => {
            let mut branches = 0;
            for edge in &edges {
                branches += emit_edge_target(
                    content,
                    node_id,
                    edge,
                    spec_path,
                    spec_refs,
                    lines,
                    inner_indent,
                    depth + 1,
                    visiting,
                    loop_info,
                    active_continuations,
                );
            }
            branches
        }
        FlowNodeKind::Process => {
            let section_id = format!("{}-{}", content.id, node_id.replace('-', "_"));
            let marker = emit_spec_ref(
                spec_path,
                &section_id,
                &format!(
                    "Implement process step: {}",
                    node.label.as_deref().unwrap_or(node_id)
                ),
                Lang::Rust,
            );
            for marker_line in marker.lines() {
                lines.push(format!("{}{}", inner_pad, marker_line));
            }
            spec_refs.push(format!("{}#{}", spec_path, section_id));

            if let Some(emitted) = emit_primitive_for_node(node) {
                for emitted_line in emitted.lines() {
                    lines.push(format!("{}{}", inner_pad, emitted_line));
                }
            } else {
                lines.push(format!(
                    "{}{};",
                    inner_pad,
                    label_to_process(
                        node.fn_name.as_deref(),
                        node.label.as_deref(),
                        node_id,
                        &node.params,
                        node.is_async,
                    )
                ));
            }

            let mut branches = 0;
            for edge in &edges {
                branches += emit_edge_target(
                    content,
                    node_id,
                    edge,
                    spec_path,
                    spec_refs,
                    lines,
                    inner_indent,
                    depth + 1,
                    visiting,
                    loop_info,
                    active_continuations,
                );
            }
            branches
        }
        FlowNodeKind::Decision => {
            let label_display = node.label.as_deref().unwrap_or(node_id);
            let condition =
                label_to_condition(node.fn_name.as_deref(), node.label.as_deref(), node_id);
            lines.push(format!("{}// Decision: {}", inner_pad, label_display));

            let mut branches = 0;
            for (i, edge) in edges.iter().enumerate() {
                let branch_label = edge.label.as_deref().unwrap_or("branch");
                if i == 0 {
                    lines.push(format!(
                        "{}if {} /* {} */ {{",
                        inner_pad, condition, branch_label
                    ));
                } else if i + 1 == edges.len() {
                    lines.push(format!("{}}} else {{ /* {} */", inner_pad, branch_label));
                } else {
                    lines.push(format!(
                        "{}}} else if todo!(\"decision branch: {{}}\", {:?}) {{ /* {} */",
                        inner_pad, branch_label, branch_label
                    ));
                }

                let lines_before = lines.len();
                let refs_before = spec_refs.len();
                branches += emit_edge_target(
                    content,
                    node_id,
                    edge,
                    spec_path,
                    spec_refs,
                    lines,
                    inner_indent + 1,
                    depth + 1,
                    visiting,
                    loop_info,
                    active_continuations,
                );

                let has_content = lines.len() > lines_before;
                let has_refs = spec_refs.len() > refs_before;
                if !has_content && !has_refs {
                    let section_id =
                        format!("{}-{}-{}", content.id, node_id, edge.to.replace('-', "_"),);
                    let marker = emit_spec_ref(
                        spec_path,
                        &section_id,
                        &format!("Implement branch {} -> {}", node_id, edge.to),
                        Lang::Rust,
                    );
                    for marker_line in marker.lines() {
                        lines.push(format!("{}    {}", inner_pad, marker_line));
                    }
                    spec_refs.push(format!("{}#{}", spec_path, section_id));
                }

                let has_stmt = lines[lines_before..].iter().any(|l| {
                    let t = l.trim_start();
                    t.starts_with("return ")
                        || t.starts_with("break")
                        || t.starts_with("continue")
                        || t.starts_with("todo!(")
                        || t.ends_with(';')
                });
                if !has_stmt {
                    lines.push(format!("{}    todo!()", inner_pad));
                }

                branches += 1;
            }
            if !edges.is_empty() {
                lines.push(format!("{}}}", inner_pad));
            }
            branches
        }
        FlowNodeKind::Terminal => {
            let stmt = label_to_terminal(node.label.as_deref(), node_id);
            lines.push(format!("{}{};", inner_pad, stmt));
            0
        }
    };

    // Close the loop opening if this was a loop head, then flush deferred
    // cycle-exit continuations outside the loop's scope.
    if is_loop_head {
        lines.push(format!("{}}}", pad));
        // Drain inner_continuations into actual emissions outside the loop.
        let to_emit: Vec<String> = std::mem::take(&mut inner_continuations);
        for cont_id in to_emit {
            // Cycle-exit continuations may legitimately re-enter the cycle's
            // outer scope; they are emitted at `indent`, not `inner_indent`.
            emit_node(
                content,
                &cont_id,
                spec_path,
                spec_refs,
                lines,
                indent,
                depth + 1,
                visiting,
                loop_info,
                continuations,
            );
        }
    }

    visiting.remove(node_id);
    branches
}

/// Follow one outgoing edge with cycle-aware semantics.
///
/// Resolution order:
///  - target ∈ visiting (DFS stack contains target)             → emit `continue;`
///  - target ∉ visiting AND from is inside a loop AND target is
///    NOT in the same loop's SCC                                → emit `break;`
///                                                                + defer
///  - otherwise                                                 → recurse into target
///
/// /// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/logic-cycle-detection.md#logic (edge handling)
#[allow(clippy::too_many_arguments)]
fn emit_edge_target(
    content: &LogicContent,
    from_node: &str,
    edge: &crate::generate::diagrams::content::logic::FlowEdge,
    spec_path: &str,
    spec_refs: &mut Vec<String>,
    lines: &mut Vec<String>,
    indent: usize,
    depth: usize,
    visiting: &mut HashSet<String>,
    loop_info: &LoopInfo,
    continuations: &mut Vec<String>,
) -> usize {
    let pad = "    ".repeat(indent);

    if visiting.contains(&edge.to) {
        lines.push(format!("{}continue;", pad));
        return 0;
    }

    let from_loop = loop_info.node_to_loop.get(from_node);
    let to_loop = loop_info.node_to_loop.get(&edge.to);
    if from_loop.is_some() && from_loop != to_loop {
        lines.push(format!("{}break;", pad));
        if !continuations.contains(&edge.to) {
            continuations.push(edge.to.clone());
        }
        return 0;
    }

    emit_node(
        content,
        &edge.to,
        spec_path,
        spec_refs,
        lines,
        indent,
        depth,
        visiting,
        loop_info,
        continuations,
    )
}

// CODEGEN-END

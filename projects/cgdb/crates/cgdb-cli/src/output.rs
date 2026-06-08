// HANDWRITE-BEGIN gap="missing-generator:hand-written:f97b8059" tracker="2087" reason="Dual output renderer — --format json (default non-tty) and --format mermaid (default tty) for query results."
use anyhow::Result;
use cgdb_core::rpc::RpcResponse;
use serde_json::Value;

use crate::OutputFormat;

pub fn render(method: &str, resp: &RpcResponse, fmt: OutputFormat) -> Result<()> {
    let body = resp.result.clone().unwrap_or(Value::Null);
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&body)?);
        }
        OutputFormat::Mermaid => match method {
            "query.coverage" => render_coverage_mermaid(&body),
            "query.impact" => render_impact_mermaid(&body),
            _ => println!("{}", serde_json::to_string_pretty(&body)?),
        },
    }
    Ok(())
}

fn render_coverage_mermaid(body: &Value) {
    let spec_total = body.get("spec_total").and_then(|v| v.as_u64()).unwrap_or(0);
    let code_total = body.get("code_total").and_then(|v| v.as_u64()).unwrap_or(0);
    let spec_orphans = body
        .get("spec_orphans")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0) as u64;
    let code_orphans = body
        .get("code_orphans")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0) as u64;
    println!("```mermaid");
    println!("---");
    println!("title: cgdb coverage");
    println!("---");
    println!("%%{{init: {{'theme': 'default'}}}}%%");
    println!("flowchart LR");
    println!(
        "  S[\"Spec nodes: {} ({} orphans)\"]",
        spec_total, spec_orphans
    );
    println!(
        "  C[\"Code nodes: {} ({} orphans)\"]",
        code_total, code_orphans
    );
    println!("  S --- C");
    println!("```");
}

fn render_impact_mermaid(body: &Value) {
    let selector = body.get("spec_section").and_then(|v| v.as_str()).unwrap_or("");
    println!("```mermaid");
    println!("---");
    println!("title: cgdb impact — {}", selector);
    println!("---");
    println!("flowchart TD");
    println!("  spec[\"{}\"]", selector);
    if let Some(arr) = body.get("affected").and_then(|v| v.as_array()) {
        for (idx, item) in arr.iter().enumerate() {
            let file = item.get("file").and_then(|v| v.as_str()).unwrap_or("");
            let symbol = item.get("symbol").and_then(|v| v.as_str()).unwrap_or("");
            println!("  n{}[\"{}::{}\"]", idx, file, symbol);
            println!("  n{} --> spec", idx);
        }
    }
    println!("```");
}
// HANDWRITE-END
// SPEC-MANAGED: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#changes

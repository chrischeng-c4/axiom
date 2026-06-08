// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/examples/audit_smoke.md#tests
// CODEGEN-BEGIN

//! Ad-hoc smoke test for the audit primitives. Run with
//! `cargo run -q -p sdd --example audit_smoke`.

fn main() {
    use agentic_workflow::generate::audit::{audit_file, audit_markers, ReportKind};
    use std::path::Path;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("projects/sdd should live under <repo>/projects/sdd");
    let files = [
        "projects/mamba/mambalibs/httpkit/src/http_exception.rs",
        "projects/mamba/mambalibs/httpkit/src/health.rs",
        "projects/mamba/mambalibs/httpkit/src/request_response.rs",
        "projects/mamba/mambalibs/httpkit/src/lib.rs",
        "projects/httpkit-demo/src/create_user_request.rs",
        "projects/httpkit-demo/src/lib.rs",
    ];

    let mut total_blocks = 0;
    let mut clean = 0;
    let mut drift = 0;
    let mut aggregate = 0;
    let mut unresolvable = 0;
    let mut gap_count = 0;

    for f in files {
        let target = root.join(f);
        match audit_file(&target, root) {
            Ok(reports) => {
                for r in &reports {
                    total_blocks += 1;
                    match &r.kind {
                        ReportKind::Clean => clean += 1,
                        ReportKind::Drift { diff } => {
                            drift += 1;
                            println!("[DRIFT]        {} {}: {}", f, r.spec_ref, diff);
                        }
                        ReportKind::Aggregate => aggregate += 1,
                        ReportKind::Unresolvable { reason } => {
                            unresolvable += 1;
                            println!("[UNRESOLVABLE] {} {}: {}", f, r.spec_ref, reason);
                        }
                    }
                }
            }
            Err(e) => eprintln!("[ERROR] audit_file {}: {}", f, e),
        }

        match audit_markers(&target) {
            Ok(gaps) => {
                gap_count += gaps.len();
                for g in &gaps {
                    println!(
                        "[GAP]          {}:{} in {} — item `{}`",
                        f, g.line_no, g.enclosing_spec_ref, g.item_line
                    );
                }
            }
            Err(e) => eprintln!("[ERROR] audit_markers {}: {}", f, e),
        }
    }

    println!(
        "{} blocks: {} clean, {} drift, {} aggregate, {} unresolvable; {} marker gaps",
        total_blocks, clean, drift, aggregate, unresolvable, gap_count,
    );
}
// CODEGEN-END

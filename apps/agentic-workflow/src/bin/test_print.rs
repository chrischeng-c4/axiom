use agentic_workflow::cli::capability::resolve_capability_path;
use std::path::Path;

fn main() {
    let root = Path::new("/Users/chrischeng/axiom/project-mamba");
    match resolve_capability_path(root, "mamba", None) {
        Ok(path) => {
            println!("Resolved capability path for mamba: {}", path.display());
        }
        Err(e) => {
            println!("Error resolving capability path: {:?}", e);
        }
    }
}

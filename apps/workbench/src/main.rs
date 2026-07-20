// HANDWRITE-BEGIN gap="missing-generator:logic:268d068b" tracker="pending-tracker" reason="Launch the native desktop host without defining a second agent CLI or session surface."
//! Workbench desktop process entrypoint.

/// @spec apps/workbench/tech-design/interfaces/rest/bootstrap-workbench-product-contract-and-runnable-desktop-applic.md#logic
fn main() {
    if let Err(error) = workbench::run() {
        eprintln!("failed to run Workbench desktop host: {error}");
        std::process::exit(1);
    }
}
// HANDWRITE-END

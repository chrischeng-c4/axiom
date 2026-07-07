use anyhow::Result;
use axiom_client_transport_policy_example::{
    h2_physical_connection_hint, lumen_client_transport_samples, EXAMPLE_TARGET_CONCURRENCY,
};

fn main() -> Result<()> {
    println!("lumen OpenAPI generated client transport policy:");
    for sample in lumen_client_transport_samples()? {
        println!(
            "- {}: files={}, transport_policy={}",
            sample.language, sample.file_count, sample.transport_contract
        );
    }
    println!(
        "h2 physical connection hint for target_concurrency={}: {}",
        EXAMPLE_TARGET_CONCURRENCY,
        h2_physical_connection_hint()
    );
    Ok(())
}

//! Example: Generate JSON schemas for task types
//!
//! Run with:
//! ```bash
//! cargo run -p cclab-meteor --features "schema" --example generate_schemas
//! ```

#[cfg(feature = "schema")]
use cclab_queue::schema::{generate_all_schemas, generate_asyncapi};

#[cfg(feature = "schema")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Generating JSON Schemas for cclab-meteor ===\n");

    let schemas = generate_all_schemas();

    for (name, schema) in schemas.iter() {
        println!("--- {} Schema ---", name);
        println!("{}\n", serde_json::to_string_pretty(&schema)?);
    }

    println!("=== AsyncAPI Specification ===");
    let asyncapi = generate_asyncapi()?;
    // Print first 1000 chars to verify template loads
    let preview_len = 1000.min(asyncapi.len());
    println!("{}", &asyncapi[..preview_len]);
    if asyncapi.len() > 1000 {
        println!("... ({} more characters)", asyncapi.len() - 1000);
    }

    Ok(())
}

#[cfg(not(feature = "schema"))]
fn main() {
    eprintln!("Error: This example requires the 'schema' feature.");
    eprintln!("Run with: cargo run -p cclab-meteor --features schema --example generate_schemas");
    std::process::exit(1);
}

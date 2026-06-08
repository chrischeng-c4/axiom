// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-examples-payload-database-example-rs.md#source
// CODEGEN-BEGIN
use meter::{PayloadCategory, PayloadDatabase};

fn main() {
    println!("=== PayloadDatabase Example ===\n");

    let db = PayloadDatabase::new();

    // Show all categories
    println!("Available payload categories:");
    let categories = [
        PayloadCategory::SqlInjection,
        PayloadCategory::NoSqlInjection,
        PayloadCategory::PathTraversal,
        PayloadCategory::CommandInjection,
        PayloadCategory::LdapInjection,
        PayloadCategory::TemplateInjection,
        PayloadCategory::IdentifierInjection,
        PayloadCategory::UnicodeTricks,
        PayloadCategory::Overflow,
    ];

    for category in &categories {
        let payloads = db.by_category(*category);
        println!("  {:?}: {} payloads", category, payloads.len());
    }

    println!("\n=== Sample Payloads ===\n");

    // NoSQL injection samples
    println!("NoSQL Injection (first 5):");
    for (i, payload) in db.nosql_injection().iter().take(5).enumerate() {
        println!("  {}. {}", i + 1, payload);
    }

    // Command injection samples
    println!("\nCommand Injection (first 5):");
    for (i, payload) in db.command_injection().iter().take(5).enumerate() {
        println!("  {}. {}", i + 1, payload);
    }

    // Template injection samples
    println!("\nTemplate Injection (first 5):");
    for (i, payload) in db.template_injection().iter().take(5).enumerate() {
        println!("  {}. {}", i + 1, payload);
    }

    // Path traversal samples
    println!("\nPath Traversal (first 5):");
    for (i, payload) in db.path_traversal().iter().take(5).enumerate() {
        println!("  {}. {}", i + 1, payload);
    }

    // LDAP injection samples
    println!("\nLDAP Injection (first 5):");
    for (i, payload) in db.ldap_injection().iter().take(5).enumerate() {
        println!("  {}. {}", i + 1, payload);
    }

    println!("\nTotal payloads: {}", db.all().len());

    // Demonstrate category usage
    println!("\n=== Category-based Access ===\n");
    let nosql_category = PayloadCategory::NoSqlInjection;
    let nosql_payloads = db.by_category(nosql_category);
    println!(
        "Accessing {:?} via by_category(): {} payloads",
        nosql_category,
        nosql_payloads.len()
    );
}
// CODEGEN-END

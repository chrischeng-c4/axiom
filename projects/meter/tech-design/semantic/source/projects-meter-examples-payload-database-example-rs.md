---
id: projects-meter-examples-payload-database-example-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: legacy-carried-internals
    role: primary
    gap: seeded-fuzz-and-injection-finding-generation
    claim: seeded-fuzz-and-injection-finding-generation
    coverage: full
    rationale: "Source template implements meter security, fuzzing, injection, or audit surfaces."
---

# Standardized projects/meter/examples/payload_database_example.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/examples/payload_database_example.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/examples/payload_database_example.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/examples/payload_database_example.rs` captured during meter full-codegen standardization.
```

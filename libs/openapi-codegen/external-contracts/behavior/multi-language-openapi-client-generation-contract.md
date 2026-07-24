---
id: multi-language-openapi-client-generation-contract
summary: External contract for Multi-Language OpenAPI Client Generation.
fill_sections: [e2e-test]
---

# EC: Multi-Language OpenAPI Client Generation

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: python-311-generated-model-contract
    capability_id: multi-language-openapi-client-generation
    claim_id: multi-language-openapi-client-generation-contract
    contract_id: python-311-generated-model-contract
    category: behavior
    command: "cargo test -p cclab-openapi-codegen --test target_profile_matrix python_311_generated_model_contract -- --exact"
    assertions:
      - "Python 3.11 output compiles, imports with Pydantic 2.12.5, and validates the generated Pet model under a fail-closed uv-managed Python 3.11 runtime."
      - "The materialized Python 3.11 sidecar is read back from disk and exactly declares python, minimum version 3.11, and the ordered pydantic>=2 runtime dependency."
  - id: python-312-generated-model-contract
    capability_id: multi-language-openapi-client-generation
    claim_id: multi-language-openapi-client-generation-contract
    contract_id: python-312-generated-model-contract
    category: behavior
    command: "cargo test -p cclab-openapi-codegen --test target_profile_matrix python_312_generated_model_contract -- --exact"
    assertions:
      - "Python 3.12 output uses a PEP 695 alias, compiles, imports, and validates the generated Pet model under Python 3.12."
      - "A missing Python runtime, dependency, generated file, import, or model-validation result fails the case."
  - id: python-313-generated-model-contract
    capability_id: multi-language-openapi-client-generation
    claim_id: multi-language-openapi-client-generation-contract
    contract_id: python-313-generated-model-contract
    category: behavior
    command: "cargo test -p cclab-openapi-codegen --test target_profile_matrix python_313_generated_model_contract -- --exact"
    assertions:
      - "Python 3.13 output uses target-valid typing, compiles, imports, and validates the generated Pet model under Python 3.13."
      - "The observable gen wire property and optional tag field survive Pydantic validation and serialization."
  - id: python-314-generated-model-contract
    capability_id: multi-language-openapi-client-generation
    claim_id: multi-language-openapi-client-generation-contract
    contract_id: python-314-generated-model-contract
    category: behavior
    command: "cargo test -p cclab-openapi-codegen --test target_profile_matrix python_314_generated_model_contract -- --exact"
    assertions:
      - "Python 3.14 output uses target-valid typing, compiles, imports, and validates the generated Pet model under Python 3.14."
      - "The output carries exact Python 3.14 target requirements rather than inheriting an implicit language default."
  - id: typescript-50-strict-modern-module-contract
    capability_id: multi-language-openapi-client-generation
    claim_id: multi-language-openapi-client-generation-contract
    contract_id: typescript-50-strict-modern-module-contract
    category: behavior
    command: "cargo test -p cclab-openapi-codegen --test target_profile_matrix typescript_50_strict_modern_module_contract -- --exact"
    assertions:
      - "Generated TypeScript artifacts have the exact types/runtime/client/index file set and type-check with TypeScript 5.0.4, target ES2022, module ESNext, moduleResolution Bundler, strict, and verbatimModuleSyntax."
      - "An independent consumer imports Pet in type position and createClient in value position, while the parsed on-disk manifest matches the tsc compiler/module/strictness contract."
  - id: rust-2021-generated-client-contract
    capability_id: multi-language-openapi-client-generation
    claim_id: multi-language-openapi-client-generation-contract
    contract_id: rust-2021-generated-client-contract
    category: behavior
    command: "cargo test -p cclab-openapi-codegen --test target_profile_matrix rust_2021_generated_client_contract -- --exact"
    assertions:
      - "The exact models/client/mod file set compiles and runs an independent Pet consumer as a temporary Cargo edition 2021 crate with its declared dependencies."
      - "The independent consumer constructs Pet.gen and proves serde serialization/deserialization preserves the external JSON key gen."
  - id: rust-2024-generated-client-gen-property-contract
    capability_id: multi-language-openapi-client-generation
    claim_id: multi-language-openapi-client-generation-contract
    contract_id: rust-2024-generated-client-gen-property-contract
    category: behavior
    command: "cargo test -p cclab-openapi-codegen --test target_profile_matrix rust_2024_generated_client_gen_property_contract -- --exact"
    assertions:
      - "The exact models/client/mod file set compiles and runs an independent Pet consumer as a temporary Cargo edition 2024 crate."
      - "The consumer constructs Pet.gen_, round-trips it through serde, and proves JSON contains gen and never gen_."
  - id: legacy-default-output-contract
    capability_id: multi-language-openapi-client-generation
    claim_id: multi-language-openapi-client-generation-contract
    contract_id: legacy-default-output-contract
    category: behavior
    command: "cargo test -p cclab-openapi-codegen --test target_profile_matrix legacy_default_output_contract -- --exact"
    assertions:
      - "GenOptions target None reproduces fixed file lists and byte fingerprints for legacy TypeScript, Python, and Rust generation."
      - "TypeScript, Python, and Rust legacy outputs are each materialized independently and each proves the target sidecar is absent."
  - id: deterministic-target-requirements-and-artifacts-contract
    capability_id: multi-language-openapi-client-generation
    claim_id: multi-language-openapi-client-generation-contract
    contract_id: deterministic-target-requirements-and-artifacts-contract
    category: behavior
    command: "cargo test -p cclab-openapi-codegen --test target_profile_matrix all_target_requirements_and_artifacts_are_deterministic -- --exact"
    assertions:
      - "All seven declared profiles reproduce identical ordered files, file bytes, target requirements, runtime-dependency order, and parsed on-disk manifest values across repeated runs."
      - "Every targeted output is materialized twice and the two sidecar files are byte-identical after exact field validation."
  - id: multi-language-openapi-client-generation-contract
    capability_id: multi-language-openapi-client-generation
    claim_id: multi-language-openapi-client-generation-contract
    contract_id: multi-language-openapi-client-generation-contract
    category: behavior
    command: "cargo test -p cclab-openapi-codegen --test target_profile_matrix"
    assertions:
      - "The historical umbrella contract remains stable and executes all nine fail-closed target-profile matrix cases, so no profile-specific case may disappear unnoticed."
      - "The full external matrix covers Python 3.11-3.14 model smoke, TypeScript strict consumer type-check, Rust edition consumers, legacy golden compatibility, and deterministic materialized sidecars."
```

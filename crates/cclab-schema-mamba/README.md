# Cclab Schema Mamba

## Brief

Cclab Schema Mamba is the Mamba native binding for `cclab-schema` dataclass and
model validation primitives.

It registers the primary `mambalibs.dataclasses` module plus the legacy
`cclab_schema_mamba` compatibility alias. The binding owns model definition
surfaces such as `BaseModel`, `DataClass`, `Field`, `create_model`, and bound
method getters, plus validation, JSON parsing, model dumping, JSON Schema
generation, alias/default/coercion handling, nested model support, and
framework-facing validation detail helpers.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Mamba Dataclass Model Definition Binding | - | implemented | passing | conformance | not_ready | defines BaseModel/DataClass/Field/create_model surface for Mamba dataclass-style models |
| Mamba Model Validation Dump And Schema Binding | - | implemented | passing | conformance | not_ready | validates, normalizes, dumps, parses JSON, and emits JSON Schema through the Mamba binding |

### Mamba Dataclass Model Definition Binding

ID: mamba-dataclass-model-definition-binding
Type: DeveloperTool
Surfaces: Mamba module: `mambalibs.dataclasses`; Compatibility module: `cclab_schema_mamba`; Native ABI: `BaseModel`, `DataClass`, `Field`, `create_model`, `add_field`, `add_fields`, bound method getters
EC Dimensions: behavior: `cargo test -p cclab-schema-mamba`
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Schema Mamba exposes `cclab-schema` model definition primitives to Mamba through the `mambalibs.dataclasses` module, with a compatibility alias, BaseModel/DataClass constructors, Field descriptors, dynamic model creation, field registration, and bound model method dispatch.
Gate Inventory: `cargo test -p cclab-schema-mamba`; crates/cclab-schema-mamba/src/lib.rs; crates/cclab-schema-mamba/src/types.rs; crates/cclab-schema-mamba/tests/test_binding.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Mamba dataclass model definition contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-schema-mamba`; crates/cclab-schema-mamba/src/lib.rs; crates/cclab-schema-mamba/src/types.rs; crates/cclab-schema-mamba/tests/test_binding.rs |

### Mamba Model Validation Dump And Schema Binding

ID: mamba-model-validation-dump-and-schema-binding
Type: DeveloperTool
Surfaces: Mamba module: `mambalibs.dataclasses`; Native ABI: `validate`, `model_validate`, `parse_obj`, `model_validate_json`, `parse_raw`, `model_dump`, `model_dump_json`, `to_json_schema`, `model_json_schema`, `field_validator`
EC Dimensions: behavior: `cargo test -p cclab-schema-mamba`
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Schema Mamba exposes validated model execution to Mamba scripts, including boolean validation, normalized dict output, JSON input parsing, compact JSON dumping, JSON Schema generation, aliases/defaults/coercion, nested models, and framework-facing validation detail helpers.
Gate Inventory: `cargo test -p cclab-schema-mamba`; crates/cclab-schema-mamba/src/methods.rs; crates/cclab-schema-mamba/src/types.rs; crates/cclab-schema-mamba/tests/test_binding.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Mamba model validation dump and schema contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-schema-mamba`; crates/cclab-schema-mamba/src/methods.rs; crates/cclab-schema-mamba/src/types.rs; crates/cclab-schema-mamba/tests/test_binding.rs |

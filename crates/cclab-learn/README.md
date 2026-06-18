# Cclab Learn

## Brief

Cclab Learn is a Rust machine-learning library with a default classical ML
surface and an optional deep-learning surface.

The default `ml` feature exposes scikit-learn-like estimator and transformer
traits, regression/classification/clustering models, preprocessing, metrics,
splitting, pipelines, cross-validation, and grid search. The `dl` / `full`
feature adds flat-data tensors, tape-based autograd, neural-network layers,
optimizers, dataloaders, recurrent and attention layers, and model-weight
serialization.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Classical ML Estimator Toolkit | - | implemented | passing | conformance | not_ready | default `ml` feature exposes scikit-learn-like estimators, transformers, metrics, and workflow utilities |
| Deep Learning Tensor And Layer Toolkit | - | implemented | passing | conformance | not_ready | `full`/`dl` feature exposes tensor autograd, layers, optimizers, data loading, and model-weight serialization |

### Classical ML Estimator Toolkit

ID: classical-ml-estimator-toolkit
Type: DeveloperTool
Surfaces: Rust API: `cclab_learn::ml`; Cargo feature: `ml`
EC Dimensions: behavior: `cargo test -p cclab-learn`
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Learn provides a Rust-native scikit-learn-like ML API with shared estimator/transformer traits, classical supervised and unsupervised models, preprocessing and feature engineering helpers, metrics, dataset splitting, pipelines, cross-validation, and grid search.
Gate Inventory: `cargo test -p cclab-learn`; crates/cclab-learn/src/ml/mod.rs; crates/cclab-learn/src/ml/traits.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Classical ML estimator and workflow contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-learn`; crates/cclab-learn/src/ml/mod.rs; crates/cclab-learn/src/ml/traits.rs |

### Deep Learning Tensor And Layer Toolkit

ID: deep-learning-tensor-and-layer-toolkit
Type: DeveloperTool
Surfaces: Rust API: `cclab_learn::dl`; Cargo features: `dl` / `full`
EC Dimensions: behavior: `cargo test -p cclab-learn --features full`
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Learn provides a feature-gated Rust deep-learning toolkit with flat-data tensors, tape-based automatic differentiation, neural-network layers and activations, optimizers, recurrent and attention layers, dataloaders, and JSON/binary model-weight serialization.
Gate Inventory: `cargo test -p cclab-learn --features full`; crates/cclab-learn/src/dl/mod.rs; crates/cclab-learn/src/dl/tensor.rs; crates/cclab-learn/src/dl/serialization.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Deep learning tensor and layer contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-learn --features full`; crates/cclab-learn/src/dl/mod.rs; crates/cclab-learn/src/dl/tensor.rs; crates/cclab-learn/src/dl/serialization.rs |

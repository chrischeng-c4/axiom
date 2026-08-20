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

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Classical ML Estimator Toolkit | - | default `ml` feature exposes scikit-learn-like estimators, transformers, metrics, and workflow utilities |
| Deep Learning Tensor And Layer Toolkit | - | `full`/`dl` feature exposes tensor autograd, layers, optimizers, data loading, and model-weight serialization |

### Classical ML Estimator Toolkit

Cclab Learn provides a Rust-native scikit-learn-like ML API with shared
estimator/transformer traits, classical supervised and unsupervised models,
preprocessing and feature engineering helpers, metrics, dataset splitting,
pipelines, cross-validation, and grid search.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `cclab_learn::ml`; Cargo feature: `ml`
- Gate — behavior: `cargo test -p cclab-learn`
- Gate: `cargo test -p cclab-learn`
- Source: `crates/cclab-learn/src/ml/mod.rs`,
  `crates/cclab-learn/src/ml/traits.rs`
- Evidence: `cargo test -p cclab-learn`; crates/cclab-learn/src/ml/mod.rs;
  crates/cclab-learn/src/ml/traits.rs

### Deep Learning Tensor And Layer Toolkit

Cclab Learn provides a feature-gated Rust deep-learning toolkit with flat-data
tensors, tape-based automatic differentiation, neural-network layers and
activations, optimizers, recurrent and attention layers, dataloaders, and
JSON/binary model-weight serialization.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `cclab_learn::dl`; Cargo features: `dl` / `full`
- Gate — behavior: `cargo test -p cclab-learn --features full`
- Gate: `cargo test -p cclab-learn --features full`
- Source: `crates/cclab-learn/src/dl/mod.rs`,
  `crates/cclab-learn/src/dl/tensor.rs`,
  `crates/cclab-learn/src/dl/serialization.rs`
- Evidence: `cargo test -p cclab-learn --features full`;
  crates/cclab-learn/src/dl/mod.rs; crates/cclab-learn/src/dl/tensor.rs;
  crates/cclab-learn/src/dl/serialization.rs

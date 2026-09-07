//! Colocated tests for [`crate::core`], the interpreter-free half of the
//! binding.
//!
//! These run under `cargo test -p scikit-pyo3 --lib` with no Python present:
//! the module registry, the refusal messages, and the argument checks that
//! keep a `scikit` assertion from becoming a panic across the FFI boundary
//! are all plain Rust, so they are judged here rather than only through the
//! wheel the e2e case builds.
//!
//! Every name below is prefixed `sci_`. `arraykit-pyo3` publishes its own
//! `tests::` module in the same workspace, and the impl phase keys a test
//! result by its bare `module::function` path, so a name shared with that
//! crate would be reported twice under one key.

use crate::core::{
    checked_dense, checked_dot, checked_point_cloud, checked_point_pair, checked_query_point,
    submodule_path, SciError, MODULE_NAME, SUBMODULES,
};
use scikit::optimize::OptimizeError;
use scikit::ts::TsError;

#[test]
fn sci_publishes_the_nine_modules_of_the_full_feature_set() {
    assert_eq!(
        SUBMODULES.to_vec(),
        vec![
            "stats",
            "fft",
            "signal",
            "interpolate",
            "optimize",
            "ts",
            "spatial",
            "sparse",
            "integrate",
        ]
    );
}

#[test]
fn sci_names_each_submodule_under_the_extension_module() {
    assert_eq!(MODULE_NAME, "mambalibs.sci");
    for name in SUBMODULES {
        assert_eq!(submodule_path(name), format!("mambalibs.sci.{name}"));
    }
}

#[test]
fn sci_an_optimize_refusal_keeps_the_reason_scikit_gave() {
    let err = SciError::from(OptimizeError::InvalidBounds(
        "f(a) and f(b) must have opposite signs".into(),
    ));
    assert!(
        err.message().contains("opposite signs"),
        "{}",
        err.message()
    );
}

#[test]
fn sci_a_timeseries_refusal_keeps_the_reason_scikit_gave() {
    let err = SciError::from(TsError::InsufficientData { need: 2, got: 1 });
    assert!(err.message().contains("at least 2"), "{}", err.message());
    assert!(err.message().contains("got 1"), "{}", err.message());
}

#[test]
fn sci_two_points_of_different_rank_cannot_be_measured() {
    assert_eq!(checked_point_pair(&[0.0, 0.0], &[3.0, 4.0]), Ok(()));
    let err = checked_point_pair(&[0.0, 0.0], &[1.0]).expect_err("rank mismatch");
    assert!(err.message().contains('2') && err.message().contains('1'), "{}", err.message());
}

#[test]
fn sci_a_point_cloud_must_be_non_empty_and_of_one_rank() {
    let cloud = vec![vec![0.0, 0.0], vec![1.0, 1.0], vec![2.0, 2.0]];
    assert_eq!(checked_point_cloud(&cloud), Ok(2));

    assert!(checked_point_cloud(&[]).is_err());
    assert!(checked_point_cloud(&[vec![]]).is_err());
    let ragged = vec![vec![0.0, 0.0], vec![1.0]];
    let err = checked_point_cloud(&ragged).expect_err("ragged cloud");
    assert!(err.message().contains("point 1"), "{}", err.message());
}

#[test]
fn sci_a_query_point_must_match_the_rank_of_its_tree() {
    assert_eq!(checked_query_point(2, &[0.1, 0.1]), Ok(()));
    assert!(checked_query_point(2, &[0.1]).is_err());
    assert!(checked_query_point(2, &[0.1, 0.1, 0.1]).is_err());
}

#[test]
fn sci_a_dense_buffer_must_hold_exactly_one_element_per_cell() {
    assert_eq!(checked_dense(4, 2, 2), Ok(()));
    assert_eq!(checked_dense(0, 0, 5), Ok(()));
    let err = checked_dense(3, 2, 2).expect_err("short buffer");
    assert!(err.message().contains('4') && err.message().contains('3'), "{}", err.message());
}

#[test]
fn sci_dense_dimensions_that_overflow_are_refused_not_wrapped() {
    let err = checked_dense(0, usize::MAX, 2).expect_err("overflowing product");
    assert!(err.message().contains("address"), "{}", err.message());
}

#[test]
fn sci_a_matrix_vector_product_needs_one_element_per_column() {
    assert_eq!(checked_dot(2, 2), Ok(()));
    let err = checked_dot(2, 3).expect_err("wrong length vector");
    assert!(err.message().contains("2 columns"), "{}", err.message());
}

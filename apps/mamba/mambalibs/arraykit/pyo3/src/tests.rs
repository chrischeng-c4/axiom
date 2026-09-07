//! Colocated tests for the dtype-erased core in [`crate::core`].
//!
//! These run without a Python interpreter: every rule the wheel's behaviour
//! rests on — dtype mapping, row-major geometry, element coercion, and the
//! refusal of an out-of-range index — lives in plain Rust, so it is judged
//! here rather than only through the wheel the e2e case builds.

use crate::core::{shape_of, Array, CoreError, Scalar, ALL_DTYPES};
use arraykit::DType;

#[test]
fn the_five_dtypes_carry_their_numpy_names_and_sizes() {
    let seen: Vec<(&str, usize)> = ALL_DTYPES.iter().map(|d| (d.numpy_str(), d.size())).collect();
    assert_eq!(
        seen,
        vec![
            ("float32", 4),
            ("float64", 8),
            ("int32", 4),
            ("int64", 8),
            ("bool", 1),
        ]
    );
}

#[test]
fn shape_is_row_major() {
    let shape = shape_of(vec![2, 3, 4]);
    assert_eq!(shape.ndim(), 3);
    assert_eq!(shape.dims(), &[2, 3, 4]);
    assert_eq!(shape.strides(), &[12, 4, 1]);
    assert_eq!(shape.size(), 24);
}

#[test]
fn a_rank_zero_shape_holds_one_element() {
    let shape = shape_of(vec![]);
    assert_eq!(shape.ndim(), 0);
    assert_eq!(shape.dims(), &[] as &[usize]);
    assert_eq!(shape.size(), 1);
}

#[test]
fn zeros_reports_its_geometry_for_every_dtype() {
    for dtype in ALL_DTYPES {
        let arr = Array::zeros(vec![2, 3], dtype);
        assert_eq!(arr.dtype(), dtype, "{dtype}");
        assert_eq!(arr.ndim(), 2, "{dtype}");
        assert_eq!(arr.dims(), vec![2, 3], "{dtype}");
        assert_eq!(arr.size(), 6, "{dtype}");
    }
}

#[test]
fn zeros_ones_and_full_fill_every_element() {
    let zeros = Array::zeros(vec![3], DType::Int64);
    let ones = Array::ones(vec![3], DType::Int64);
    let full = Array::full(vec![3], DType::Int32, Scalar::Int(7)).expect("full");
    for i in 0..3 {
        assert_eq!(zeros.get(&[i]).unwrap(), Scalar::Int(0));
        assert_eq!(ones.get(&[i]).unwrap(), Scalar::Int(1));
        assert_eq!(full.get(&[i]).unwrap(), Scalar::Int(7));
    }
}

#[test]
fn each_dtype_reads_back_as_its_own_scalar_kind() {
    assert_eq!(
        Array::ones(vec![1], DType::Float32).get(&[0]).unwrap(),
        Scalar::Float(1.0)
    );
    assert_eq!(
        Array::ones(vec![1], DType::Float64).get(&[0]).unwrap(),
        Scalar::Float(1.0)
    );
    assert_eq!(
        Array::ones(vec![1], DType::Int32).get(&[0]).unwrap(),
        Scalar::Int(1)
    );
    assert_eq!(
        Array::ones(vec![1], DType::Bool).get(&[0]).unwrap(),
        Scalar::Bool(true)
    );
    assert_eq!(
        Array::zeros(vec![1], DType::Bool).get(&[0]).unwrap(),
        Scalar::Bool(false)
    );
}

#[test]
fn set_addresses_one_element_and_leaves_the_rest() {
    let mut arr = Array::zeros(vec![2, 2], DType::Int32);
    arr.set(&[1, 0], Scalar::Int(5)).expect("set");
    assert_eq!(arr.get(&[1, 0]).unwrap(), Scalar::Int(5));
    assert_eq!(arr.get(&[0, 1]).unwrap(), Scalar::Int(0));
    assert_eq!(arr.get(&[0, 0]).unwrap(), Scalar::Int(0));
    assert_eq!(arr.get(&[1, 1]).unwrap(), Scalar::Int(0));
}

#[test]
fn an_out_of_range_index_is_an_error_not_a_silent_value() {
    let arr = Array::zeros(vec![2], DType::Int32);
    assert!(matches!(arr.get(&[5]), Err(CoreError::Index(_))));
    assert!(matches!(arr.get(&[0, 0]), Err(CoreError::Index(_))));

    let mut arr = Array::zeros(vec![2], DType::Int32);
    assert!(matches!(
        arr.set(&[5], Scalar::Int(1)),
        Err(CoreError::Index(_))
    ));
}

#[test]
fn reshape_keeps_the_element_count_and_refuses_a_change_of_it() {
    let arr = Array::zeros(vec![2, 6], DType::Float32);
    let reshaped = arr.reshape(vec![3, 4]).expect("reshape");
    assert_eq!(reshaped.dims(), vec![3, 4]);
    assert_eq!(reshaped.size(), arr.size());
    assert_eq!(reshaped.dtype(), DType::Float32);
    assert!(matches!(arr.reshape(vec![5, 5]), Err(CoreError::Shape(_))));
}

#[test]
fn flatten_gives_rank_one_and_transpose_reverses_the_dimensions() {
    let arr = Array::zeros(vec![2, 6], DType::Float32);
    let flat = arr.flatten();
    assert_eq!(flat.ndim(), 1);
    assert_eq!(flat.dims(), vec![12]);

    let arr = Array::zeros(vec![2, 3], DType::Float64);
    assert_eq!(arr.transpose().dims(), vec![3, 2]);
    assert_eq!(arr.transpose().dtype(), DType::Float64);
}

#[test]
fn a_scalar_widens_the_way_python_numbers_do() {
    assert_eq!(Scalar::Bool(true).as_f64(), 1.0);
    assert_eq!(Scalar::Int(3).as_f64(), 3.0);
    assert_eq!(Scalar::Float(2.5).as_f64(), 2.5);

    assert_eq!(Scalar::Bool(true).as_i64().unwrap(), 1);
    assert_eq!(Scalar::Int(-4).as_i64().unwrap(), -4);
    assert_eq!(Scalar::Float(6.0).as_i64().unwrap(), 6);

    assert!(!Scalar::Int(0).as_bool());
    assert!(Scalar::Int(2).as_bool());
    assert!(!Scalar::Float(0.0).as_bool());
}

#[test]
fn a_fractional_value_is_refused_by_an_integer_array_rather_than_truncated() {
    assert!(matches!(
        Scalar::Float(2.5).as_i64(),
        Err(CoreError::Value(_))
    ));
    assert!(matches!(
        Array::full(vec![2], DType::Int32, Scalar::Float(2.5)),
        Err(CoreError::Value(_))
    ));
    let mut arr = Array::zeros(vec![2], DType::Int64);
    assert!(matches!(
        arr.set(&[0], Scalar::Float(0.5)),
        Err(CoreError::Value(_))
    ));
}

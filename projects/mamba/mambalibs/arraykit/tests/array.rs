//! Integration tests for the array module.

use arraykit::array::{AxisSlice, DType, NdArray, Shape, SliceInfo};

// ============================================================================
// DType tests (from dtype.rs)
// ============================================================================

#[test]
fn test_dtype_size() {
    assert_eq!(DType::Float32.size(), 4);
    assert_eq!(DType::Float64.size(), 8);
    assert_eq!(DType::Int32.size(), 4);
    assert_eq!(DType::Int64.size(), 8);
    assert_eq!(DType::Bool.size(), 1);
}

#[test]
fn test_dtype_display() {
    assert_eq!(format!("{}", DType::Float32), "float32");
    assert_eq!(format!("{}", DType::Int64), "int64");
}

// ============================================================================
// Shape tests (from shape.rs)
// ============================================================================

#[test]
fn test_shape_creation() {
    let shape = Shape::new(vec![2, 3, 4]);
    assert_eq!(shape.ndim(), 3);
    assert_eq!(shape.dims(), &[2, 3, 4]);
    assert_eq!(shape.strides(), &[12, 4, 1]);
    assert_eq!(shape.size(), 24);
}

#[test]
fn test_shape_offset() {
    let shape = Shape::new(vec![2, 3]);
    assert_eq!(shape.offset(&[0, 0]).unwrap(), 0);
    assert_eq!(shape.offset(&[0, 1]).unwrap(), 1);
    assert_eq!(shape.offset(&[1, 0]).unwrap(), 3);
    assert_eq!(shape.offset(&[1, 2]).unwrap(), 5);
}

#[test]
fn test_broadcast_same_shape() {
    let a = Shape::new(vec![2, 3]);
    let b = Shape::new(vec![2, 3]);
    let result = Shape::broadcast(&a, &b).unwrap();
    assert_eq!(result.dims(), &[2, 3]);
}

#[test]
fn test_broadcast_scalar() {
    let a = Shape::new(vec![2, 3]);
    let b = Shape::new(vec![1]);
    let result = Shape::broadcast(&a, &b).unwrap();
    assert_eq!(result.dims(), &[2, 3]);
}

#[test]
fn test_broadcast_different_ndim() {
    let a = Shape::new(vec![2, 3]);
    let b = Shape::new(vec![3]);
    let result = Shape::broadcast(&a, &b).unwrap();
    assert_eq!(result.dims(), &[2, 3]);
}

#[test]
fn test_broadcast_error() {
    let a = Shape::new(vec![2, 3]);
    let b = Shape::new(vec![4]);
    let result = Shape::broadcast(&a, &b);
    assert!(result.is_err());
}

// ============================================================================
// Slice tests (from slice.rs)
// ============================================================================

#[test]
fn test_axis_slice_full() {
    let slice = AxisSlice::Full;
    let norm = slice.normalize(5).unwrap();
    assert_eq!(norm.start, 0);
    assert_eq!(norm.stop, 5);
    assert_eq!(norm.step, 1);
    assert_eq!(norm.output_len, 5);
}

#[test]
fn test_axis_slice_range() {
    let slice = AxisSlice::range(1, 4);
    let norm = slice.normalize(5).unwrap();
    assert_eq!(norm.start, 1);
    assert_eq!(norm.stop, 4);
    assert_eq!(norm.step, 1);
    assert_eq!(norm.output_len, 3);
}

#[test]
fn test_axis_slice_negative_index() {
    let slice = AxisSlice::range(-2, -1);
    let norm = slice.normalize(5).unwrap();
    assert_eq!(norm.start, 3); // 5 + (-2) = 3
    assert_eq!(norm.stop, 4); // 5 + (-1) = 4
    assert_eq!(norm.step, 1);
    assert_eq!(norm.output_len, 1);
}

#[test]
fn test_axis_slice_negative_step() {
    // arr[4:1:-1] should give indices 4, 3, 2
    let slice = AxisSlice::range_step(4, 1, -1);
    let norm = slice.normalize(5).unwrap();
    assert_eq!(norm.start, 4);
    assert_eq!(norm.stop, 1);
    assert_eq!(norm.step, -1);
    assert_eq!(norm.output_len, 3);
}

#[test]
fn test_axis_slice_reverse() {
    // arr[::-1] should reverse the array
    let slice = AxisSlice::Slice {
        start: None,
        stop: None,
        step: Some(-1),
    };
    let norm = slice.normalize(5).unwrap();
    assert_eq!(norm.start, 4); // Default start for negative step
    assert_eq!(norm.stop, -1isize); // Before index 0 (isize)
    assert_eq!(norm.step, -1);
    assert_eq!(norm.output_len, 5);
}

#[test]
fn test_slice_info_output_shape() {
    let info = SliceInfo::new(vec![AxisSlice::Full, AxisSlice::Index(1)]);
    let shape = info.output_shape(&[3, 4]).unwrap();
    assert_eq!(shape, vec![3]); // Index reduces dimension
}

#[test]
fn test_slice_column() {
    // Slice to get second column: arr[:, 1]
    let info = SliceInfo::new(vec![AxisSlice::Full, AxisSlice::Index(1)]);
    let shape = info.output_shape(&[2, 2]).unwrap();
    assert_eq!(shape, vec![2]); // 2x2 -> 2 (column vector)
}

// ============================================================================
// NdArray tests (from ndarray.rs)
// ============================================================================

#[test]
fn test_create_2d_array() {
    let arr = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    assert_eq!(arr.dims(), &[2, 2]);
    assert_eq!(arr.get(&[0, 0]).unwrap(), 1);
    assert_eq!(arr.get(&[0, 1]).unwrap(), 2);
    assert_eq!(arr.get(&[1, 0]).unwrap(), 3);
    assert_eq!(arr.get(&[1, 1]).unwrap(), 4);
}

#[test]
fn test_zeros_ones() {
    let zeros = NdArray::<f64>::zeros(vec![2, 3]);
    assert_eq!(zeros.size(), 6);
    assert_eq!(zeros.get(&[0, 0]).unwrap(), 0.0);

    let ones = NdArray::<i32>::ones(vec![3, 2]);
    assert_eq!(ones.get(&[1, 1]).unwrap(), 1);
}

#[test]
fn test_reshape() {
    let arr = NdArray::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let reshaped = arr.reshape(vec![3, 2]).unwrap();
    assert_eq!(reshaped.dims(), &[3, 2]);
    assert_eq!(reshaped.get(&[0, 0]).unwrap(), 1);
    assert_eq!(reshaped.get(&[2, 1]).unwrap(), 6);
}

#[test]
fn test_ndarray_slice_column() {
    // Test acceptance scenario: slice 2D array to get second column
    let arr = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let info = SliceInfo::new(vec![AxisSlice::Full, AxisSlice::Index(1)]);
    let result = arr.slice(&info).unwrap();
    assert_eq!(result.dims(), &[2]);
    assert_eq!(result.data(), &[2, 4]);
}

#[test]
fn test_ndarray_slice_reverse() {
    // Test reverse slicing: arr[::-1]
    let arr = NdArray::new(vec![1, 2, 3, 4, 5], vec![5]).unwrap();
    let reverse_slice = AxisSlice::Slice {
        start: None,
        stop: None,
        step: Some(-1),
    };
    let info = SliceInfo::new(vec![reverse_slice]);
    let result = arr.slice(&info).unwrap();
    assert_eq!(result.dims(), &[5]);
    assert_eq!(result.data(), &[5, 4, 3, 2, 1]);
}

#[test]
fn test_arange_positive_step() {
    let arr = NdArray::arange(0, 5, 1);
    assert_eq!(arr.data(), &[0, 1, 2, 3, 4]);
}

#[test]
fn test_arange_negative_step() {
    let arr = NdArray::arange(5, 0, -1);
    assert_eq!(arr.data(), &[5, 4, 3, 2, 1]);
}

#[test]
fn test_arange_zero_step() {
    let arr = NdArray::arange(0, 5, 0);
    assert_eq!(arr.size(), 0); // Empty array for step=0
}

#[test]
fn test_transpose_2d() {
    // [[1, 2, 3], [4, 5, 6]] -> [[1, 4], [2, 5], [3, 6]]
    let arr = NdArray::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let transposed = arr.transpose();
    assert_eq!(transposed.dims(), &[3, 2]);
    assert_eq!(transposed.get(&[0, 0]).unwrap(), 1);
    assert_eq!(transposed.get(&[0, 1]).unwrap(), 4);
    assert_eq!(transposed.get(&[1, 0]).unwrap(), 2);
    assert_eq!(transposed.get(&[1, 1]).unwrap(), 5);
    assert_eq!(transposed.get(&[2, 0]).unwrap(), 3);
    assert_eq!(transposed.get(&[2, 1]).unwrap(), 6);
}

#[test]
fn test_transpose_1d() {
    // 1D transpose should return the same array
    let arr = NdArray::new(vec![1, 2, 3], vec![3]).unwrap();
    let transposed = arr.transpose();
    assert_eq!(transposed.dims(), &[3]);
    assert_eq!(transposed.data(), &[1, 2, 3]);
}

#[test]
fn test_transpose_axes() {
    // 2x3x4 -> permute(2, 0, 1) -> 4x2x3
    let data: Vec<i32> = (0..24).collect();
    let arr = NdArray::new(data, vec![2, 3, 4]).unwrap();
    let transposed = arr.transpose_axes(&[2, 0, 1]).unwrap();
    assert_eq!(transposed.dims(), &[4, 2, 3]);
}

#[test]
fn test_eye() {
    let eye: NdArray<f64> = NdArray::eye(3);
    assert_eq!(eye.dims(), &[3, 3]);
    assert_eq!(eye.get(&[0, 0]).unwrap(), 1.0);
    assert_eq!(eye.get(&[1, 1]).unwrap(), 1.0);
    assert_eq!(eye.get(&[2, 2]).unwrap(), 1.0);
    assert_eq!(eye.get(&[0, 1]).unwrap(), 0.0);
    assert_eq!(eye.get(&[1, 0]).unwrap(), 0.0);
}

#[test]
fn test_identity() {
    let id: NdArray<i32> = NdArray::identity(2);
    assert_eq!(id.dims(), &[2, 2]);
    assert_eq!(id.data(), &[1, 0, 0, 1]);
}

#[test]
fn test_diag_from_1d() {
    let vec: NdArray<i32> = NdArray::new(vec![1, 2, 3], vec![3]).unwrap();
    let diag = NdArray::diag(&vec).unwrap();
    assert_eq!(diag.dims(), &[3, 3]);
    assert_eq!(diag.get(&[0, 0]).unwrap(), 1);
    assert_eq!(diag.get(&[1, 1]).unwrap(), 2);
    assert_eq!(diag.get(&[2, 2]).unwrap(), 3);
    assert_eq!(diag.get(&[0, 1]).unwrap(), 0);
}

#[test]
fn test_diag_from_2d() {
    let mat: NdArray<i32> = NdArray::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9], vec![3, 3]).unwrap();
    let diag = NdArray::diag(&mat).unwrap();
    assert_eq!(diag.dims(), &[3]);
    assert_eq!(diag.data(), &[1, 5, 9]);
}

#[test]
fn test_full() {
    let arr: NdArray<i32> = NdArray::full(vec![2, 3], 5);
    assert_eq!(arr.size(), 6);
    assert_eq!(arr.get(&[0, 0]).unwrap(), 5);
    assert_eq!(arr.get(&[1, 2]).unwrap(), 5);
}

#[test]
fn test_shape() {
    let arr = NdArray::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let shape = arr.shape();
    assert_eq!(shape.dims(), &[2, 3]);
}

#[test]
fn test_dims_ndim_size() {
    let arr = NdArray::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    assert_eq!(arr.dims(), &[2, 3]);
    assert_eq!(arr.ndim(), 2);
    assert_eq!(arr.size(), 6);
}

#[test]
fn test_data_data_mut() {
    let mut arr = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    assert_eq!(arr.data(), &[1, 2, 3, 4]);

    arr.data_mut()[0] = 10;
    assert_eq!(arr.data(), &[10, 2, 3, 4]);
}

#[test]
fn test_get_set() {
    let mut arr = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    assert_eq!(arr.get(&[0, 1]).unwrap(), 2);

    arr.set(&[0, 1], 20).unwrap();
    assert_eq!(arr.get(&[0, 1]).unwrap(), 20);
}

#[test]
fn test_flatten() {
    let arr = NdArray::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let flat = arr.flatten();
    assert_eq!(flat.dims(), &[6]);
    assert_eq!(flat.data(), &[1, 2, 3, 4, 5, 6]);
}

#[test]
fn test_broadcast_to() {
    let arr = NdArray::new(vec![1, 2, 3], vec![3]).unwrap();
    let target = Shape::new(vec![2, 3]);
    let broadcasted = arr.broadcast_to(&target).unwrap();
    assert_eq!(broadcasted.dims(), &[2, 3]);
    assert_eq!(broadcasted.get(&[0, 0]).unwrap(), 1);
    assert_eq!(broadcasted.get(&[1, 0]).unwrap(), 1);
}

// ============================================================================
// Ops tests (from ops.rs)
// ============================================================================

#[test]
fn test_add_same_shape() {
    let a = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let b = NdArray::new(vec![10, 20, 30, 40], vec![2, 2]).unwrap();
    let c = a.add(&b).unwrap();
    assert_eq!(c.data(), &[11, 22, 33, 44]);
}

#[test]
fn test_broadcast_add() {
    // Test acceptance scenario: 1D + 2D broadcasting
    let a = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let b = NdArray::new(vec![10], vec![1]).unwrap();
    let c = a.add(&b).unwrap();
    assert_eq!(c.dims(), &[2, 2]);
    assert_eq!(c.data(), &[11, 12, 13, 14]);
}

#[test]
fn test_broadcast_row() {
    let a = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let b = NdArray::new(vec![10, 20], vec![2]).unwrap();
    let c = a.add(&b).unwrap();
    assert_eq!(c.data(), &[11, 22, 13, 24]);
}

#[test]
fn test_sub() {
    let a = NdArray::new(vec![10, 20, 30, 40], vec![2, 2]).unwrap();
    let b = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let c = a.sub(&b).unwrap();
    assert_eq!(c.data(), &[9, 18, 27, 36]);
}

#[test]
fn test_mul() {
    let a = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let b = NdArray::new(vec![2, 3, 4, 5], vec![2, 2]).unwrap();
    let c = a.mul(&b).unwrap();
    assert_eq!(c.data(), &[2, 6, 12, 20]);
}

#[test]
fn test_div() {
    let a = NdArray::new(vec![10.0, 20.0, 30.0, 40.0], vec![2, 2]).unwrap();
    let b = NdArray::new(vec![2.0, 4.0, 5.0, 8.0], vec![2, 2]).unwrap();
    let c = a.div(&b).unwrap();
    assert_eq!(c.data(), &[5.0, 5.0, 6.0, 5.0]);
}

#[test]
fn test_scalar_ops() {
    let a = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let b = a.add_scalar(10);
    assert_eq!(b.data(), &[11, 12, 13, 14]);

    let c = a.mul_scalar(2);
    assert_eq!(c.data(), &[2, 4, 6, 8]);
}

#[test]
fn test_operator_syntax() {
    let a = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let b = NdArray::new(vec![10, 20, 30, 40], vec![2, 2]).unwrap();
    let c = (&a + &b).unwrap();
    assert_eq!(c.data(), &[11, 22, 33, 44]);
}

// ============================================================================
// Linalg tests (from linalg.rs)
// ============================================================================

#[test]
fn test_matmul_2x2() {
    let a: NdArray<i32> = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let b: NdArray<i32> = NdArray::new(vec![5, 6, 7, 8], vec![2, 2]).unwrap();
    let c = a.matmul(&b).unwrap();
    assert_eq!(c.dims(), &[2, 2]);
    assert_eq!(c.data(), &[19, 22, 43, 50]);
}

#[test]
fn test_matmul_2x3_3x2() {
    let a: NdArray<i32> = NdArray::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let b: NdArray<i32> = NdArray::new(vec![7, 8, 9, 10, 11, 12], vec![3, 2]).unwrap();
    let c = a.matmul(&b).unwrap();
    assert_eq!(c.dims(), &[2, 2]);
    // [1*7+2*9+3*11, 1*8+2*10+3*12] = [58, 64]
    // [4*7+5*9+6*11, 4*8+5*10+6*12] = [139, 154]
    assert_eq!(c.data(), &[58, 64, 139, 154]);
}

#[test]
fn test_dot_1d() {
    let a: NdArray<i32> = NdArray::new(vec![1, 2, 3], vec![3]).unwrap();
    let b: NdArray<i32> = NdArray::new(vec![4, 5, 6], vec![3]).unwrap();
    let c = a.dot(&b).unwrap();
    assert_eq!(c.data(), &[32]); // 1*4 + 2*5 + 3*6 = 32
}

#[test]
fn test_dot_matrix_vector() {
    let a: NdArray<i32> = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let b: NdArray<i32> = NdArray::new(vec![5, 6], vec![2]).unwrap();
    let c = a.dot(&b).unwrap();
    assert_eq!(c.dims(), &[2]);
    assert_eq!(c.data(), &[17, 39]); // [1*5+2*6, 3*5+4*6]
}

#[test]
fn test_outer() {
    let a: NdArray<i32> = NdArray::new(vec![1, 2, 3], vec![3]).unwrap();
    let b: NdArray<i32> = NdArray::new(vec![4, 5], vec![2]).unwrap();
    let c = a.outer(&b).unwrap();
    assert_eq!(c.dims(), &[3, 2]);
    assert_eq!(c.data(), &[4, 5, 8, 10, 12, 15]);
}

#[test]
fn test_trace() {
    let a: NdArray<i32> = NdArray::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9], vec![3, 3]).unwrap();
    assert_eq!(a.trace().unwrap(), 15); // 1 + 5 + 9
}

#[test]
fn test_det_2x2() {
    let a: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let det = a.det().unwrap();
    assert!(f64::abs(det - (-2.0)) < 1e-10); // 1*4 - 2*3 = -2
}

#[test]
fn test_det_3x3() {
    let a: NdArray<f64> = NdArray::new(
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0],
        vec![3, 3],
    )
    .unwrap();
    let det = a.det().unwrap();
    assert!(f64::abs(det - (-3.0)) < 1e-10);
}

#[test]
fn test_inv_2x2() {
    let a: NdArray<f64> = NdArray::new(vec![4.0, 7.0, 2.0, 6.0], vec![2, 2]).unwrap();
    let inv = a.inv().unwrap();
    // A * A^-1 should be identity
    let product = a.matmul(&inv).unwrap();
    assert!(f64::abs(product.get(&[0, 0]).unwrap() - 1.0) < 1e-10);
    assert!(f64::abs(product.get(&[0, 1]).unwrap()) < 1e-10);
    assert!(f64::abs(product.get(&[1, 0]).unwrap()) < 1e-10);
    assert!(f64::abs(product.get(&[1, 1]).unwrap() - 1.0) < 1e-10);
}

#[test]
fn test_solve() {
    // Solve: 2x + y = 5, x + 3y = 7
    let a: NdArray<f64> = NdArray::new(vec![2.0, 1.0, 1.0, 3.0], vec![2, 2]).unwrap();
    let b: NdArray<f64> = NdArray::new(vec![5.0, 7.0], vec![2]).unwrap();
    let x = a.solve(&b).unwrap();
    // x = 8/5 = 1.6, y = 9/5 = 1.8
    assert!(f64::abs(x.data()[0] - 1.6) < 1e-10);
    assert!(f64::abs(x.data()[1] - 1.8) < 1e-10);
}

#[test]
fn test_norm() {
    let a: NdArray<f64> = NdArray::new(vec![3.0, 4.0], vec![2]).unwrap();
    assert_eq!(a.norm(Some(2.0)), 5.0); // sqrt(9+16)
    assert_eq!(a.norm(Some(1.0)), 7.0); // |3| + |4|
    assert_eq!(a.norm(Some(f64::INFINITY)), 4.0); // max
}

// ============================================================================
// Math tests (from math.rs)
// ============================================================================

#[test]
fn test_linspace() {
    let arr: NdArray<f64> = NdArray::linspace(0.0, 1.0, 5);
    assert_eq!(arr.size(), 5);
    assert!(f64::abs(arr.data()[0] - 0.0) < 1e-10);
    assert!(f64::abs(arr.data()[4] - 1.0) < 1e-10);
    assert!(f64::abs(arr.data()[2] - 0.5) < 1e-10);
}

#[test]
fn test_linspace_single() {
    let arr: NdArray<f64> = NdArray::linspace(5.0, 5.0, 1);
    assert_eq!(arr.size(), 1);
    assert_eq!(arr.data()[0], 5.0);
}

#[test]
fn test_sqrt() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 4.0, 9.0, 16.0], vec![4]).unwrap();
    let result = arr.sqrt();
    assert_eq!(result.data(), &[1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_exp_log() {
    let arr: NdArray<f64> = NdArray::new(vec![0.0, 1.0, 2.0], vec![3]).unwrap();
    let exp_arr = arr.exp();
    assert!(f64::abs(exp_arr.data()[0] - 1.0) < 1e-10);
    assert!(f64::abs(exp_arr.data()[1] - std::f64::consts::E) < 1e-10);

    let log_arr = exp_arr.log();
    assert!(f64::abs(log_arr.data()[0] - 0.0) < 1e-10);
    assert!(f64::abs(log_arr.data()[1] - 1.0) < 1e-10);
}

#[test]
fn test_abs() {
    let arr: NdArray<f64> = NdArray::new(vec![-1.0, 2.0, -3.0, 4.0], vec![4]).unwrap();
    let result = arr.abs();
    assert_eq!(result.data(), &[1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_clip() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 5.0, 10.0, 15.0], vec![4]).unwrap();
    let clipped = arr.clip(3.0, 12.0);
    assert_eq!(clipped.data(), &[3.0, 5.0, 10.0, 12.0]);
}

#[test]
fn test_where_cond() {
    let a: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
    let b: NdArray<f64> = NdArray::new(vec![10.0, 20.0, 30.0, 40.0], vec![4]).unwrap();
    let cond = vec![true, false, true, false];
    let result = a.where_cond(&cond, &b).unwrap();
    assert_eq!(result.data(), &[1.0, 20.0, 3.0, 40.0]);
}

#[test]
fn test_pow() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let result = arr.pow(2.0);
    assert_eq!(result.data(), &[1.0, 4.0, 9.0]);
}

#[test]
fn test_trig() {
    let arr: NdArray<f64> = NdArray::new(vec![0.0], vec![1]).unwrap();
    assert!(f64::abs(arr.sin().data()[0] - 0.0) < 1e-10);
    assert!(f64::abs(arr.cos().data()[0] - 1.0) < 1e-10);
    assert!(f64::abs(arr.tan().data()[0] - 0.0) < 1e-10);
}

#[test]
fn test_abs_int() {
    let arr: NdArray<i32> = NdArray::new(vec![-1, 2, -3, 4], vec![4]).unwrap();
    let result = arr.abs_int();
    assert_eq!(result.data(), &[1, 2, 3, 4]);
}

// ============================================================================
// Manip tests (from manip.rs)
// ============================================================================

#[test]
fn test_concatenate_1d() {
    let a: NdArray<i32> = NdArray::new(vec![1, 2, 3], vec![3]).unwrap();
    let b: NdArray<i32> = NdArray::new(vec![4, 5, 6], vec![3]).unwrap();
    let c = NdArray::concatenate(&[&a, &b], 0).unwrap();
    assert_eq!(c.dims(), &[6]);
    assert_eq!(c.data(), &[1, 2, 3, 4, 5, 6]);
}

#[test]
fn test_concatenate_2d_axis0() {
    let a: NdArray<i32> = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let b: NdArray<i32> = NdArray::new(vec![5, 6, 7, 8], vec![2, 2]).unwrap();
    let c = NdArray::concatenate(&[&a, &b], 0).unwrap();
    assert_eq!(c.dims(), &[4, 2]);
    assert_eq!(c.data(), &[1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn test_stack() {
    let a: NdArray<i32> = NdArray::new(vec![1, 2, 3], vec![3]).unwrap();
    let b: NdArray<i32> = NdArray::new(vec![4, 5, 6], vec![3]).unwrap();
    let c = NdArray::stack(&[&a, &b], 0).unwrap();
    assert_eq!(c.dims(), &[2, 3]);
    assert_eq!(c.data(), &[1, 2, 3, 4, 5, 6]);
}

#[test]
fn test_vstack() {
    let a: NdArray<i32> = NdArray::new(vec![1, 2], vec![2]).unwrap();
    let b: NdArray<i32> = NdArray::new(vec![3, 4], vec![2]).unwrap();
    let c = NdArray::vstack(&[&a, &b]).unwrap();
    assert_eq!(c.dims(), &[4]);
    assert_eq!(c.data(), &[1, 2, 3, 4]);
}

#[test]
fn test_split() {
    let arr: NdArray<i32> = NdArray::new(vec![1, 2, 3, 4, 5, 6], vec![6]).unwrap();
    let parts = arr.split(&[2, 4], 0).unwrap();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0].data(), &[1, 2]);
    assert_eq!(parts[1].data(), &[3, 4]);
    assert_eq!(parts[2].data(), &[5, 6]);
}

#[test]
fn test_repeat() {
    let arr: NdArray<i32> = NdArray::new(vec![1, 2, 3], vec![3]).unwrap();
    let repeated = arr.repeat(2);
    assert_eq!(repeated.data(), &[1, 1, 2, 2, 3, 3]);
}

#[test]
fn test_tile() {
    let arr: NdArray<i32> = NdArray::new(vec![1, 2], vec![2]).unwrap();
    let tiled = arr.tile(&[3]).unwrap();
    assert_eq!(tiled.dims(), &[6]);
    assert_eq!(tiled.data(), &[1, 2, 1, 2, 1, 2]);
}

#[test]
fn test_unique() {
    let arr: NdArray<i32> = NdArray::new(vec![1, 2, 2, 3, 3, 3, 1], vec![7]).unwrap();
    let uniq = arr.unique();
    assert_eq!(uniq.size(), 3);
}

#[test]
fn test_sort() {
    let arr: NdArray<i32> = NdArray::new(vec![3, 1, 4, 1, 5, 9, 2, 6], vec![8]).unwrap();
    let sorted = arr.sort();
    assert_eq!(sorted.data(), &[1, 1, 2, 3, 4, 5, 6, 9]);
}

#[test]
fn test_argsort() {
    let arr: NdArray<i32> = NdArray::new(vec![3, 1, 4], vec![3]).unwrap();
    let indices = arr.argsort();
    assert_eq!(indices, vec![1, 0, 2]); // indices that sort: 1, 3, 4
}

#[test]
fn test_all_any() {
    let arr = NdArray::new(vec![true, true, true], vec![3]).unwrap();
    assert!(arr.all());
    assert!(arr.any());

    let arr2 = NdArray::new(vec![true, false, true], vec![3]).unwrap();
    assert!(!arr2.all());
    assert!(arr2.any());

    let arr3 = NdArray::new(vec![false, false, false], vec![3]).unwrap();
    assert!(!arr3.all());
    assert!(!arr3.any());
}

#[test]
fn test_allclose() {
    let a: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let b: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    assert!(a.allclose(&b, 1e-5, 1e-8));

    let c: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.1], vec![3]).unwrap();
    assert!(!a.allclose(&c, 1e-5, 1e-8));
}

// ============================================================================
// Stats tests (from stats.rs)
// ============================================================================

#[test]
fn test_sum_all() {
    let arr = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    assert_eq!(arr.sum(), 10);
}

#[test]
fn test_sum_axis_0() {
    // [[1, 2], [3, 4]] -> sum along axis 0 -> [4, 6]
    let arr = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let result = arr.sum_axis(0).unwrap();
    assert_eq!(result.dims(), &[2]);
    assert_eq!(result.data(), &[4, 6]);
}

#[test]
fn test_sum_axis_1() {
    // [[1, 2], [3, 4]] -> sum along axis 1 -> [3, 7]
    let arr = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let result = arr.sum_axis(1).unwrap();
    assert_eq!(result.dims(), &[2]);
    assert_eq!(result.data(), &[3, 7]);
}

#[test]
fn test_sum_axis_3d() {
    // 2x2x3 array
    let arr = NdArray::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], vec![2, 2, 3]).unwrap();

    // Sum along axis 2 (innermost) -> 2x2
    let result = arr.sum_axis(2).unwrap();
    assert_eq!(result.dims(), &[2, 2]);
    assert_eq!(result.data(), &[6, 15, 24, 33]); // 1+2+3, 4+5+6, 7+8+9, 10+11+12
}

#[test]
fn test_mean_all() {
    let arr = NdArray::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    assert_eq!(arr.mean(), 2.5);
}

#[test]
fn test_mean_axis() {
    let arr = NdArray::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let result = arr.mean_axis(0).unwrap();
    assert_eq!(result.dims(), &[2]);
    assert_eq!(result.data(), &[2.0, 3.0]); // (1+3)/2, (2+4)/2
}

#[test]
fn test_min_max_all() {
    let arr = NdArray::new(vec![3, 1, 4, 1, 5, 9], vec![6]).unwrap();
    assert_eq!(arr.min(), Some(1));
    assert_eq!(arr.max(), Some(9));
}

#[test]
fn test_min_axis() {
    let arr = NdArray::new(vec![3, 1, 4, 2], vec![2, 2]).unwrap();
    let result = arr.min_axis(0).unwrap();
    assert_eq!(result.dims(), &[2]);
    assert_eq!(result.data(), &[3, 1]); // min(3,4), min(1,2)
}

#[test]
fn test_max_axis() {
    let arr = NdArray::new(vec![3, 1, 4, 2], vec![2, 2]).unwrap();
    let result = arr.max_axis(1).unwrap();
    assert_eq!(result.dims(), &[2]);
    assert_eq!(result.data(), &[3, 4]); // max(3,1), max(4,2)
}

#[test]
fn test_empty_array() {
    let arr = NdArray::<f64>::zeros(vec![0]);
    assert_eq!(arr.sum(), 0.0);
    assert_eq!(arr.mean(), 0.0);
    assert_eq!(arr.min(), None);
    assert_eq!(arr.max(), None);
}

#[test]
fn test_var_std() {
    let arr = NdArray::new(vec![2.0_f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0], vec![8]).unwrap();
    let variance = arr.var();
    let std_dev = arr.std();
    // Sample variance = 4.571..., std = 2.138...
    assert!(f64::abs(variance - 4.571428571428571) < 1e-10);
    assert!(f64::abs(std_dev - 2.138089935299395) < 1e-10);
}

#[test]
fn test_var_pop() {
    let arr = NdArray::new(vec![1.0_f64, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    let var_pop = arr.var_pop();
    assert!(f64::abs(var_pop - 2.0) < 1e-10); // Population variance = 2.0
}

#[test]
fn test_argmin_argmax() {
    let arr = NdArray::new(vec![3, 1, 4, 1, 5, 9, 2, 6], vec![8]).unwrap();
    assert_eq!(arr.argmin(), Some(1)); // First occurrence of min (1)
    assert_eq!(arr.argmax(), Some(5)); // Index of max (9)
}

#[test]
fn test_cumsum() {
    let arr = NdArray::new(vec![1, 2, 3, 4], vec![4]).unwrap();
    let result = arr.cumsum();
    assert_eq!(result.data(), &[1, 3, 6, 10]);
}

#[test]
fn test_cumprod() {
    let arr = NdArray::new(vec![1, 2, 3, 4], vec![4]).unwrap();
    let result = arr.cumprod();
    assert_eq!(result.data(), &[1, 2, 6, 24]);
}

#[test]
fn test_percentile() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    assert_eq!(arr.percentile(0.0), Some(1.0));
    assert_eq!(arr.percentile(50.0), Some(3.0));
    assert_eq!(arr.percentile(100.0), Some(5.0));
}

#[test]
fn test_quantile() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    assert_eq!(arr.quantile(0.0), Some(1.0));
    assert_eq!(arr.quantile(0.5), Some(3.0));
    assert_eq!(arr.quantile(1.0), Some(5.0));
}

#[test]
fn test_corrcoef() {
    // Two variables, 5 observations each
    // x = [1, 2, 3, 4, 5], y = [2, 4, 6, 8, 10]
    let arr: NdArray<f64> = NdArray::new(
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 2.0, 4.0, 6.0, 8.0, 10.0],
        vec![2, 5],
    )
    .unwrap();
    let corr = arr.corrcoef().unwrap();
    assert_eq!(corr.dims(), &[2, 2]);
    // Perfect correlation
    assert!(f64::abs(corr.get(&[0, 1]).unwrap() - 1.0) < 1e-10);
    assert!(f64::abs(corr.get(&[1, 0]).unwrap() - 1.0) < 1e-10);
}

#[test]
fn test_cov() {
    let arr: NdArray<f64> = NdArray::new(
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 10.0, 20.0, 30.0, 40.0, 50.0],
        vec![2, 5],
    )
    .unwrap();
    let cov_mat = arr.cov().unwrap();
    assert_eq!(cov_mat.dims(), &[2, 2]);
    // Variance of [1,2,3,4,5] = 2.5
    assert!(f64::abs(cov_mat.get(&[0, 0]).unwrap() - 2.5) < 1e-10);
}

#[test]
fn test_histogram() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    let (counts, edges) = arr.histogram(5);
    assert_eq!(counts.len(), 5);
    assert_eq!(edges.len(), 6);
    assert_eq!(counts.iter().sum::<usize>(), 5);
}

// Phase 2 advanced statistics tests
#[test]
fn test_median() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 3.0, 5.0, 7.0, 9.0], vec![5]).unwrap();
    assert_eq!(arr.median(), Some(5.0));

    let arr2: NdArray<f64> = NdArray::new(vec![1.0, 3.0, 5.0, 7.0], vec![4]).unwrap();
    assert_eq!(arr2.median(), Some(4.0));
}

#[test]
fn test_mode() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0], vec![6]).unwrap();
    assert_eq!(arr.mode(), Some(3.0));
}

#[test]
fn test_skew() {
    // Symmetric distribution should have skewness near 0
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    assert!(arr.skew().abs() < 0.1);

    // Right-skewed distribution
    let arr2: NdArray<f64> = NdArray::new(vec![1.0, 1.0, 1.0, 1.0, 10.0], vec![5]).unwrap();
    assert!(arr2.skew() > 0.0);
}

#[test]
fn test_kurtosis() {
    // Normal-like distribution should have excess kurtosis near 0
    let arr: NdArray<f64> = NdArray::new(
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        vec![10],
    )
    .unwrap();
    assert!(arr.kurtosis().abs() < 2.0);
}

#[test]
fn test_zscore() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    let z = arr.zscore();
    // Mean of z-scores should be 0
    assert!(z.mean().abs() < 1e-10);
}

#[test]
fn test_iqr() {
    let arr: NdArray<f64> =
        NdArray::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0], vec![9]).unwrap();
    let result = arr.iqr().unwrap();
    assert!((result - 4.0).abs() < 0.5);
}

#[test]
fn test_geometric_mean() {
    let arr: NdArray<f64> = NdArray::new(vec![2.0, 8.0], vec![2]).unwrap();
    assert!((arr.geometric_mean() - 4.0).abs() < 1e-10);
}

#[test]
fn test_harmonic_mean() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 4.0], vec![3]).unwrap();
    let hm = arr.harmonic_mean();
    assert!((hm - 12.0 / 7.0).abs() < 1e-10);
}

#[test]
fn test_trim_mean() {
    let arr: NdArray<f64> = NdArray::new(
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        vec![10],
    )
    .unwrap();
    let tm = arr.trim_mean(0.1);
    // Should remove 1 element from each end
    assert!((tm - 5.5).abs() < 0.5);
}

#[test]
fn test_sem() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    let s = arr.sem();
    // std = sqrt(2.5), sem = sqrt(2.5) / sqrt(5)
    assert!((s - (2.5_f64.sqrt() / 5.0_f64.sqrt())).abs() < 1e-10);
}

#[test]
fn test_cv() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    let cv = arr.coefficient_of_variation();
    // mean = 3, std = sqrt(2.5), cv = sqrt(2.5) / 3
    assert!((cv - 2.5_f64.sqrt() / 3.0).abs() < 1e-10);
}

#[test]
fn test_moment() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    // Second central moment = variance (population)
    let m2 = arr.moment(2);
    assert!((m2 - 2.0).abs() < 1e-10);
}

#[test]
fn test_std_axis() {
    let arr = NdArray::new(vec![1.0_f64, 3.0, 2.0, 4.0], vec![2, 2]).unwrap();
    let result = arr.std_axis(0).unwrap();
    assert_eq!(result.dims(), &[2]);
    // std([1, 2]) and std([3, 4])
    assert!(result.data()[0] > 0.0);
}

#[test]
fn test_var_axis() {
    let arr = NdArray::new(vec![1.0_f64, 3.0, 2.0, 4.0], vec![2, 2]).unwrap();
    let result = arr.var_axis(0).unwrap();
    assert_eq!(result.dims(), &[2]);
    // var([1, 2]) and var([3, 4])
    assert!(result.data()[0] > 0.0);
}

#[test]
fn test_std_pop() {
    let arr = NdArray::new(vec![1.0_f64, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    let std_pop = arr.std_pop();
    // Population std = sqrt(2.0)
    assert!((std_pop - 2.0_f64.sqrt()).abs() < 1e-10);
}

// ============================================================================
// Window tests (from window.rs)
// ============================================================================

#[test]
fn test_rolling_sum() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    let result = arr.rolling(3).sum();
    // First 2 elements are partial windows
    assert_eq!(result.data()[2], 6.0); // 1+2+3
    assert_eq!(result.data()[3], 9.0); // 2+3+4
    assert_eq!(result.data()[4], 12.0); // 3+4+5
}

#[test]
fn test_rolling_mean() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    let result = arr.rolling(3).mean();
    assert_eq!(result.data()[2], 2.0); // (1+2+3)/3
    assert_eq!(result.data()[4], 4.0); // (3+4+5)/3
}

#[test]
fn test_rolling_min_max() {
    let arr: NdArray<f64> = NdArray::new(vec![3.0, 1.0, 4.0, 1.0, 5.0], vec![5]).unwrap();
    let min_result = arr.rolling(3).min();
    let max_result = arr.rolling(3).max();
    assert_eq!(min_result.data()[2], 1.0);
    assert_eq!(max_result.data()[2], 4.0);
}

#[test]
fn test_squeeze() {
    let arr: NdArray<i32> = NdArray::new(vec![1, 2, 3], vec![1, 3, 1]).unwrap();
    let squeezed = arr.squeeze();
    assert_eq!(squeezed.dims(), &[3]);
}

#[test]
fn test_expand_dims() {
    let arr: NdArray<i32> = NdArray::new(vec![1, 2, 3], vec![3]).unwrap();
    let expanded = arr.expand_dims(0);
    assert_eq!(expanded.dims(), &[1, 3]);
}

#[test]
fn test_flip() {
    let arr: NdArray<i32> = NdArray::new(vec![1, 2, 3, 4, 5], vec![5]).unwrap();
    let flipped = arr.flip();
    assert_eq!(flipped.data(), &[5, 4, 3, 2, 1]);
}

#[test]
fn test_roll() {
    let arr: NdArray<i32> = NdArray::new(vec![1, 2, 3, 4, 5], vec![5]).unwrap();
    let rolled = arr.roll(2);
    assert_eq!(rolled.data(), &[4, 5, 1, 2, 3]);
}

#[test]
fn test_pad() {
    let arr: NdArray<i32> = NdArray::new(vec![1, 2, 3], vec![3]).unwrap();
    let padded = arr.pad(2);
    assert_eq!(padded.data(), &[0, 0, 1, 2, 3, 0, 0]);
}

#[test]
fn test_diff() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 4.0, 7.0, 11.0], vec![5]).unwrap();
    let diff1 = arr.diff(1);
    assert_eq!(diff1.data(), &[1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_gradient() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 4.0, 7.0, 11.0], vec![5]).unwrap();
    let grad = arr.gradient();
    assert_eq!(grad.size(), 5);
    assert_eq!(grad.data()[0], 1.0); // Forward: 2-1
    assert_eq!(grad.data()[2], 2.5); // Central: (7-2)/2
}

#[test]
fn test_interp() {
    let xp = vec![0.0, 1.0, 2.0];
    let fp = vec![0.0, 10.0, 20.0];
    let x = vec![0.5, 1.5];
    let result = NdArray::interp(&x, &xp, &fp);
    assert_eq!(result, vec![5.0, 15.0]);
}

#[test]
fn test_rot90() {
    let arr: NdArray<i32> = NdArray::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let rotated = arr.rot90(1);
    assert_eq!(rotated.dims(), &[2, 2]);
    assert_eq!(rotated.data(), &[2, 4, 1, 3]);
}

#[test]
fn test_rolling_min_periods() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    // With min_periods=1, we get values even for partial windows
    let result = arr.rolling(3).min_periods(1).sum();
    assert_eq!(result.data()[0], 1.0); // just first element
    assert_eq!(result.data()[1], 3.0); // 1+2
    assert_eq!(result.data()[2], 6.0); // 1+2+3
}

#[test]
fn test_rolling_count() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    let counts = arr.rolling(3).count();
    assert_eq!(counts, vec![1, 2, 3, 3, 3]);
}

#[test]
fn test_rolling_std() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    let result = arr.rolling(3).std();
    // First two are NaN (min_periods=3 by default)
    assert!(result.data()[0].is_nan());
    assert!(result.data()[1].is_nan());
    // std of [1,2,3] = 1.0 (sample std)
    assert!((result.data()[2] - 1.0).abs() < 1e-10);
}

#[test]
fn test_rolling_var() {
    let arr: NdArray<f64> = NdArray::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();
    let result = arr.rolling(3).var();
    // First two are NaN
    assert!(result.data()[0].is_nan());
    assert!(result.data()[1].is_nan());
    // var of [1,2,3] = 1.0 (sample var)
    assert!((result.data()[2] - 1.0).abs() < 1e-10);
}

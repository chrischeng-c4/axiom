"""The Python-visible contract of the ``mambalibs.array`` extension module.

These files are fixture material, not cases: ``aw`` counts only ``.rs`` files
directly under ``e2e/`` as cases, and ``mambalibs_array_wheel.rs`` is the case
that runs this directory with ``python -m pytest -p no:cacheprovider`` inside
the environment it just built, indexed, and synced.

Every assertion below maps 1:1 onto a method that already exists on
``arraykit``'s Rust surface (``DType::size``/``numpy_str``, ``Shape::ndim``/
``dims``/``strides``/``size``, ``NdArray::zeros``/``dims``/``reshape``/...).
Nothing here asks for new behaviour in ``mambalibs/arraykit/src``; the wheel
wraps the API that is there.
"""

import pathlib

import mambalibs.array as array


# --- module identity -------------------------------------------------------


def test_module_is_a_compiled_extension():
    """The import resolves to a built extension, not a Python shim."""
    name = pathlib.Path(array.__file__).name
    assert name.startswith("array."), name
    assert name.endswith((".so", ".pyd", ".dylib")), name


def test_namespace_package_has_no_initializer():
    """PEP 420: no ``mambalibs/__init__.py``, so later kits can coexist."""
    package_dir = pathlib.Path(array.__file__).parent
    assert package_dir.name == "mambalibs", package_dir
    assert not (package_dir / "__init__.py").exists(), sorted(
        p.name for p in package_dir.iterdir()
    )


def test_public_names_are_exported():
    for name in ("NdArray", "DType", "Shape"):
        assert hasattr(array, name), sorted(dir(array))


# --- DType -----------------------------------------------------------------

DTYPES = {
    "Float32": ("float32", 4),
    "Float64": ("float64", 8),
    "Int32": ("int32", 4),
    "Int64": ("int64", 8),
    "Bool": ("bool", 1),
}


def test_dtype_has_exactly_the_five_rust_variants():
    exported = {n for n in dir(array.DType) if not n.startswith("_")}
    assert DTYPES.keys() <= exported, exported


def test_dtype_numpy_names_and_sizes():
    for variant, (numpy_name, size) in DTYPES.items():
        dtype = getattr(array.DType, variant)
        assert str(dtype) == numpy_name, variant
        assert dtype.size == size, variant


# --- Shape -----------------------------------------------------------------


def test_shape_computes_row_major_strides():
    shape = array.Shape([2, 3, 4])
    assert shape.ndim == 3
    assert list(shape.dims) == [2, 3, 4]
    assert list(shape.strides) == [12, 4, 1]
    assert shape.size == 24


def test_scalar_shape_is_rank_zero_with_size_one():
    shape = array.Shape([])
    assert shape.ndim == 0
    assert list(shape.dims) == []
    assert shape.size == 1


# --- NdArray ---------------------------------------------------------------


def test_zeros_reports_its_geometry():
    arr = array.NdArray.zeros([2, 3], array.DType.Float64)
    assert arr.ndim == 2
    assert list(arr.dims) == [2, 3]
    assert arr.size == 6
    assert str(arr.dtype) == "float64"


def test_ones_and_full_fill_every_element():
    ones = array.NdArray.ones([4], array.DType.Int64)
    assert [ones.get([i]) for i in range(4)] == [1, 1, 1, 1]

    full = array.NdArray.full([3], array.DType.Int32, 7)
    assert [full.get([i]) for i in range(3)] == [7, 7, 7]


def test_get_and_set_address_elements_by_index():
    arr = array.NdArray.zeros([2, 2], array.DType.Int32)
    arr.set([1, 0], 5)
    assert arr.get([1, 0]) == 5
    assert arr.get([0, 1]) == 0


def test_reshape_preserves_size_and_flatten_gives_rank_one():
    arr = array.NdArray.zeros([2, 6], array.DType.Float32)
    reshaped = arr.reshape([3, 4])
    assert list(reshaped.dims) == [3, 4]
    assert reshaped.size == arr.size

    flat = arr.flatten()
    assert flat.ndim == 1
    assert list(flat.dims) == [12]


def test_transpose_reverses_the_dimensions():
    arr = array.NdArray.zeros([2, 3], array.DType.Float64)
    assert list(arr.transpose().dims) == [3, 2]


def test_out_of_range_index_is_an_error_not_a_silent_value():
    arr = array.NdArray.zeros([2], array.DType.Int32)
    try:
        arr.get([5])
    except Exception:
        return
    raise AssertionError("get([5]) on a length-2 array must raise")

"""The Python-visible contract of the nine ``mambalibs.sci`` modules.

Fixture material, not a case: ``mambalibs_sci_wheel.rs`` runs this directory
with ``python -m pytest -p no:cacheprovider`` inside the environment it built.
``test_sci_namespace.py`` owns where the module lives; this file owns what it
does.

Every assertion below maps onto a function or type that already exists in
``apps/mamba/mambalibs/scikit/src``. The work item freezes "existing core API
only, no scipy-parity promise", so nothing here asks for behaviour the Rust
crate does not implement, and each expected value is derived from the Rust
source rather than from SciPy.

Naming rules
------------

The wheel is a binding, so the Python names follow the Rust ones by rule, the
same rules ``mambalibs.array`` already ships:

* a Rust free function becomes a module-level function of the same snake_case
  name, taking the same arguments in the same positional order;
* a ``pub struct`` or ``pub enum`` becomes a class of the same PascalCase name,
  and a fieldless enum publishes each variant as a class attribute
  (``ConvolveMode.Full``), exactly as ``DType.Float32`` does;
* a public field, or a zero-argument method reporting a property of the value,
  becomes a Python attribute (``TestResult.statistic``, ``Neighbor.index``,
  ``CsrMatrix.nnz``);
* a method taking arguments, or one building a new value or a different
  representation, stays a Python method (``CubicSpline.eval``,
  ``CsrMatrix.to_dense``, ``TestResult.is_significant``), as
  ``NdArray.reshape`` does;
* an associated constructor such as ``CsrMatrix::from_dense`` becomes a
  classmethod of the same name; ``Type::new`` becomes ``__init__``;
* ``Result<T, E>`` raises a Python exception, ``Option<T>`` returns ``None``,
  ``Vec<f64>`` is a ``list[float]``, and a Rust closure parameter takes any
  Python callable.

The names this contract requires
--------------------------------

* ``sci.stats``: ``median``, ``ttest_1samp``, and ``TestResult`` with
  ``.statistic``, ``.pvalue``, ``.is_significant(alpha)``.
* ``sci.fft``: ``rfftfreq``, ``rfft``, and ``Complex`` with ``.re`` / ``.im``.
* ``sci.signal``: ``hann``, ``convolve``, ``ConvolveMode.Full``.
* ``sci.interpolate``: ``interp1d``, ``InterpKind.Linear``, ``CubicSpline``
  with ``.eval(x)``.
* ``sci.optimize``: ``linprog`` returning ``LinprogResult`` with ``.x``,
  ``.fun``, ``.success``; ``brentq`` over a Python callable.
* ``sci.ts``: ``ewma``, ``acf``.
* ``sci.spatial``: ``euclidean``, ``KdTree`` with ``.query(point, k)``
  returning ``Neighbor`` values carrying ``.index`` and ``.distance``.
* ``sci.sparse``: ``CsrMatrix.from_dense``, ``.nnz``, ``.get(row, col)``,
  ``.to_dense()``, ``.dot(x)``.
* ``sci.integrate``: ``trapezoid``, ``simps``.
"""

import mambalibs.sci as sci


# --- stats -----------------------------------------------------------------


def test_stats_median_splits_odd_and_even_samples():
    """``median`` sorts first, and averages the middle pair when even."""
    assert sci.stats.median([3.0, 1.0, 2.0]) == 2.0
    assert sci.stats.median([1.0, 2.0, 3.0, 4.0]) == 2.5


def test_stats_median_of_an_empty_sample_is_none():
    """The Rust ``Option<f64>`` arrives as ``None``, not as a NaN or a raise."""
    assert sci.stats.median([]) is None


def test_stats_ttest_1samp_finds_no_difference_from_the_sample_mean():
    result = sci.stats.ttest_1samp([1.0, 2.0, 3.0], 2.0)
    assert abs(result.statistic) < 1e-12, result.statistic
    assert 0.0 <= result.pvalue <= 1.0, result.pvalue
    assert result.is_significant(0.05) is False


# --- fft -------------------------------------------------------------------


def test_fft_rfftfreq_bins_a_four_sample_window():
    assert sci.fft.rfftfreq(4, 1.0) == [0.0, 0.25, 0.5]


def test_fft_rfft_puts_the_sum_of_a_constant_signal_in_bin_zero():
    bins = sci.fft.rfft([1.0, 1.0, 1.0, 1.0])
    assert len(bins) == 3, len(bins)
    assert abs(bins[0].re - 4.0) < 1e-12, bins[0].re
    assert abs(bins[0].im) < 1e-12, bins[0].im
    assert abs(bins[1].re) < 1e-12, bins[1].re


# --- signal ----------------------------------------------------------------


def test_signal_hann_window_peaks_in_the_middle_and_is_symmetric():
    window = sci.signal.hann(5)
    assert len(window) == 5, window
    assert abs(window[0]) < 1e-12, window[0]
    assert abs(window[2] - 1.0) < 1e-12, window[2]
    assert abs(window[1] - window[3]) < 1e-12, (window[1], window[3])


def test_signal_convolve_full_mode_keeps_every_tap():
    out = sci.signal.convolve([1.0, 2.0, 3.0], [1.0, 1.0], sci.signal.ConvolveMode.Full)
    assert out == [1.0, 3.0, 5.0, 3.0], out


# --- interpolate -----------------------------------------------------------


def test_interpolate_interp1d_is_linear_between_the_knots():
    out = sci.interpolate.interp1d(
        [0.0, 1.0, 2.0],
        [0.0, 10.0, 20.0],
        [0.5, 1.5],
        sci.interpolate.InterpKind.Linear,
    )
    assert len(out) == 2, out
    assert abs(out[0] - 5.0) < 1e-12, out[0]
    assert abs(out[1] - 15.0) < 1e-12, out[1]


def test_interpolate_cubic_spline_passes_through_its_knots():
    spline = sci.interpolate.CubicSpline([0.0, 1.0, 2.0, 3.0], [0.0, 1.0, 8.0, 27.0])
    assert abs(spline.eval(1.0) - 1.0) < 1e-9, spline.eval(1.0)
    assert abs(spline.eval(2.0) - 8.0) < 1e-9, spline.eval(2.0)


# --- optimize --------------------------------------------------------------


def test_optimize_linprog_solves_a_two_variable_program():
    """Minimize -x - 2y subject to x + y <= 4, x <= 3, y <= 3, x, y >= 0."""
    result = sci.optimize.linprog(
        [-1.0, -2.0],
        [1.0, 1.0, 1.0, 0.0, 0.0, 1.0],
        [4.0, 3.0, 3.0],
    )
    assert result.success is True, result.success
    assert abs(result.x[0] - 1.0) < 1e-6, result.x
    assert abs(result.x[1] - 3.0) < 1e-6, result.x
    assert abs(result.fun + 7.0) < 1e-6, result.fun


def test_optimize_brentq_finds_a_root_of_a_python_callable():
    root = sci.optimize.brentq(lambda x: x * x - 2.0, 0.0, 2.0, 1e-12, 100)
    assert abs(root - 2.0**0.5) < 1e-6, root


def test_optimize_brentq_refuses_bounds_that_bracket_no_root():
    """The Rust ``Result`` error arm has to reach Python as an exception."""
    try:
        sci.optimize.brentq(lambda x: x * x + 1.0, 0.0, 2.0, 1e-12, 100)
    except Exception:
        return
    raise AssertionError("brentq must raise when f(a) and f(b) share a sign")


# --- ts --------------------------------------------------------------------


def test_ts_ewma_seeds_with_the_first_observation():
    assert sci.ts.ewma([1.0, 2.0, 3.0], 0.5) == [1.0, 1.5, 2.25]


def test_ts_acf_starts_at_one_and_has_one_entry_per_lag():
    values = sci.ts.acf([1.0, 2.0, 3.0, 4.0, 5.0], 2)
    assert len(values) == 3, values
    assert abs(values[0] - 1.0) < 1e-12, values[0]


def test_ts_acf_refuses_a_series_too_short_to_have_one():
    try:
        sci.ts.acf([1.0], 1)
    except Exception:
        return
    raise AssertionError("acf must raise on a series shorter than two points")


# --- spatial ---------------------------------------------------------------


def test_spatial_euclidean_measures_the_three_four_five_triangle():
    assert sci.spatial.euclidean([0.0, 0.0], [3.0, 4.0]) == 5.0


def test_spatial_kdtree_returns_neighbors_nearest_first():
    tree = sci.spatial.KdTree([[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]])
    neighbors = tree.query([0.1, 0.1], 2)
    assert len(neighbors) == 2, neighbors
    assert neighbors[0].index == 0, neighbors[0].index
    assert neighbors[0].distance < neighbors[1].distance


# --- sparse ----------------------------------------------------------------


def test_sparse_csr_from_dense_stores_only_the_nonzeros():
    matrix = sci.sparse.CsrMatrix.from_dense([1.0, 0.0, 0.0, 2.0], 2, 2)
    assert matrix.nnz == 2, matrix.nnz
    assert matrix.get(0, 0) == 1.0
    assert matrix.get(0, 1) == 0.0
    assert matrix.to_dense() == [1.0, 0.0, 0.0, 2.0]


def test_sparse_csr_multiplies_a_dense_vector():
    matrix = sci.sparse.CsrMatrix.from_dense([1.0, 0.0, 0.0, 2.0], 2, 2)
    assert matrix.dot([1.0, 1.0]) == [1.0, 2.0]


# --- integrate -------------------------------------------------------------


def test_integrate_trapezoid_of_a_ramp():
    assert sci.integrate.trapezoid([0.0, 1.0, 2.0], 1.0) == 2.0


def test_integrate_simps_is_exact_for_a_parabola():
    """Simpson's rule integrates x**2 over [0, 2] exactly: 8/3."""
    value = sci.integrate.simps([0.0, 1.0, 4.0], 1.0)
    assert abs(value - 8.0 / 3.0) < 1e-12, value

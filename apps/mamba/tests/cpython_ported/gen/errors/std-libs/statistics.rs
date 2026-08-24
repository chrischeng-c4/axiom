use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/statistics/geometric_mean_negative_raises.py`.
#[test]
fn test_gen_errors_std_libs_statistics_geometric_mean_negative_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "geometric_mean_negative_raises"
# subject = "statistics.geometric_mean"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.geometric_mean: geometric_mean_negative_raises (errors)."""
import statistics

_raised = False
try:
    statistics.geometric_mean([1, -1])
except statistics.StatisticsError:
    _raised = True
assert _raised, "geometric_mean_negative_raises: expected statistics.StatisticsError"
print("geometric_mean_negative_raises OK")
"###);
    assert_output(&out, r###"geometric_mean_negative_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/statistics/harmonic_mean_negative_raises.py`.
#[test]
fn test_gen_errors_std_libs_statistics_harmonic_mean_negative_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "harmonic_mean_negative_raises"
# subject = "statistics.harmonic_mean"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.harmonic_mean: harmonic_mean_negative_raises (errors)."""
import statistics

_raised = False
try:
    statistics.harmonic_mean([1, -2, 3])
except statistics.StatisticsError:
    _raised = True
assert _raised, "harmonic_mean_negative_raises: expected statistics.StatisticsError"
print("harmonic_mean_negative_raises OK")
"###);
    assert_output(&out, r###"harmonic_mean_negative_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/statistics/mean_empty_raises.py`.
#[test]
fn test_gen_errors_std_libs_statistics_mean_empty_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "mean_empty_raises"
# subject = "statistics.mean"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.mean: mean_empty_raises (errors)."""
import statistics

_raised = False
try:
    statistics.mean([])
except statistics.StatisticsError:
    _raised = True
assert _raised, "mean_empty_raises: expected statistics.StatisticsError"
print("mean_empty_raises OK")
"###);
    assert_output(&out, r###"mean_empty_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/statistics/mean_nonnumeric_raises.py`.
#[test]
fn test_gen_errors_std_libs_statistics_mean_nonnumeric_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "mean_nonnumeric_raises"
# subject = "statistics.mean"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.mean: mean_nonnumeric_raises (errors)."""
import statistics

_raised = False
try:
    statistics.mean([1, 2, '3'])
except TypeError:
    _raised = True
assert _raised, "mean_nonnumeric_raises: expected TypeError"
print("mean_nonnumeric_raises OK")
"###);
    assert_output(&out, r###"mean_nonnumeric_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/statistics/median_empty_raises.py`.
#[test]
fn test_gen_errors_std_libs_statistics_median_empty_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "median_empty_raises"
# subject = "statistics.median"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.median: median_empty_raises (errors)."""
import statistics

_raised = False
try:
    statistics.median([])
except statistics.StatisticsError:
    _raised = True
assert _raised, "median_empty_raises: expected statistics.StatisticsError"
print("median_empty_raises OK")
"###);
    assert_output(&out, r###"median_empty_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/statistics/mode_empty_raises.py`.
#[test]
fn test_gen_errors_std_libs_statistics_mode_empty_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "mode_empty_raises"
# subject = "statistics.mode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.mode: mode_empty_raises (errors)."""
import statistics

_raised = False
try:
    statistics.mode([])
except statistics.StatisticsError:
    _raised = True
assert _raised, "mode_empty_raises: expected statistics.StatisticsError"
print("mode_empty_raises OK")
"###);
    assert_output(&out, r###"mode_empty_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/statistics/quantiles_zero_n_raises.py`.
#[test]
fn test_gen_errors_std_libs_statistics_quantiles_zero_n_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "quantiles_zero_n_raises"
# subject = "statistics.quantiles"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.quantiles: quantiles_zero_n_raises (errors)."""
import statistics

_raised = False
try:
    statistics.quantiles([1, 2, 3], n=0)
except statistics.StatisticsError:
    _raised = True
assert _raised, "quantiles_zero_n_raises: expected statistics.StatisticsError"
print("quantiles_zero_n_raises OK")
"###);
    assert_output(&out, r###"quantiles_zero_n_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/statistics/stdev_single_raises.py`.
#[test]
fn test_gen_errors_std_libs_statistics_stdev_single_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "stdev_single_raises"
# subject = "statistics.stdev"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.stdev: stdev_single_raises (errors)."""
import statistics

_raised = False
try:
    statistics.stdev([1])
except statistics.StatisticsError:
    _raised = True
assert _raised, "stdev_single_raises: expected statistics.StatisticsError"
print("stdev_single_raises OK")
"###);
    assert_output(&out, r###"stdev_single_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/statistics/variance_single_raises.py`.
#[test]
fn test_gen_errors_std_libs_statistics_variance_single_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "variance_single_raises"
# subject = "statistics.variance"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.variance: variance_single_raises (errors)."""
import statistics

_raised = False
try:
    statistics.variance([1])
except statistics.StatisticsError:
    _raised = True
assert _raised, "variance_single_raises: expected statistics.StatisticsError"
print("variance_single_raises OK")
"###);
    assert_output(&out, r###"variance_single_raises OK
"###);
}

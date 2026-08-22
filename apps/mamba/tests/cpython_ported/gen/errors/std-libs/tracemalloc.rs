use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/tracemalloc/domain_filter_field_readonly_raises.py`.
#[test]
fn test_gen_errors_std_libs_tracemalloc_domain_filter_field_readonly_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "errors"
# case = "domain_filter_field_readonly_raises"
# subject = "tracemalloc.DomainFilter"
# kind = "mechanical"
# xfail = "mamba does not implement the tracemalloc.DomainFilter class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.DomainFilter: domain_filter_field_readonly_raises (errors)."""
import tracemalloc
_d = tracemalloc.DomainFilter(True, 5)

_raised = False
try:
    setattr(_d, 'domain', 9)
except AttributeError:
    _raised = True
assert _raised, "domain_filter_field_readonly_raises: expected AttributeError"
print("domain_filter_field_readonly_raises OK")
"###);
    assert_output(&out, r###"domain_filter_field_readonly_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tracemalloc/filter_field_readonly_raises.py`.
#[test]
fn test_gen_errors_std_libs_tracemalloc_filter_field_readonly_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "errors"
# case = "filter_field_readonly_raises"
# subject = "tracemalloc.Filter"
# kind = "mechanical"
# xfail = "mamba does not implement the tracemalloc.Filter class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Filter: filter_field_readonly_raises (errors)."""
import tracemalloc
_f = tracemalloc.Filter(True, 'abc')

_raised = False
try:
    setattr(_f, 'filename_pattern', 'x')
except AttributeError:
    _raised = True
assert _raised, "filter_field_readonly_raises: expected AttributeError"
print("filter_field_readonly_raises OK")
"###);
    assert_output(&out, r###"filter_field_readonly_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tracemalloc/filter_traces_bare_filter_raises.py`.
#[test]
fn test_gen_errors_std_libs_tracemalloc_filter_traces_bare_filter_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "errors"
# case = "filter_traces_bare_filter_raises"
# subject = "tracemalloc.Snapshot"
# kind = "mechanical"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: filter_traces_bare_filter_raises (errors)."""
import tracemalloc

_raised = False
try:
    tracemalloc.Snapshot([(0, 10, (('a.py', 2),), 1)], 1).filter_traces(tracemalloc.Filter(False, 'a.py'))
except TypeError:
    _raised = True
assert _raised, "filter_traces_bare_filter_raises: expected TypeError"
print("filter_traces_bare_filter_raises OK")
"###);
    assert_output(&out, r###"filter_traces_bare_filter_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tracemalloc/start_negative_nframe_raises.py`.
#[test]
fn test_gen_errors_std_libs_tracemalloc_start_negative_nframe_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "errors"
# case = "start_negative_nframe_raises"
# subject = "tracemalloc.start"
# kind = "mechanical"
# xfail = "mamba tracemalloc.start is a no-op shim; negative nframe is not validated, so no ValueError is raised (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.start: start_negative_nframe_raises (errors)."""
import tracemalloc

_raised = False
try:
    tracemalloc.start(-1)
except ValueError:
    _raised = True
assert _raised, "start_negative_nframe_raises: expected ValueError"
print("start_negative_nframe_raises OK")
"###);
    assert_output(&out, r###"start_negative_nframe_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tracemalloc/statistics_traceback_cumulative_raises.py`.
#[test]
fn test_gen_errors_std_libs_tracemalloc_statistics_traceback_cumulative_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "errors"
# case = "statistics_traceback_cumulative_raises"
# subject = "tracemalloc.Snapshot"
# kind = "mechanical"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: statistics_traceback_cumulative_raises (errors)."""
import tracemalloc

_raised = False
try:
    tracemalloc.Snapshot([(0, 10, (('a.py', 2),), 1)], 1).statistics('traceback', cumulative=True)
except ValueError:
    _raised = True
assert _raised, "statistics_traceback_cumulative_raises: expected ValueError"
print("statistics_traceback_cumulative_raises OK")
"###);
    assert_output(&out, r###"statistics_traceback_cumulative_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tracemalloc/take_snapshot_without_start_raises.py`.
#[test]
fn test_gen_errors_std_libs_tracemalloc_take_snapshot_without_start_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "errors"
# case = "take_snapshot_without_start_raises"
# subject = "tracemalloc.take_snapshot"
# kind = "mechanical"
# xfail = "mamba tracemalloc is a GC-counter shim; take_snapshot returns a stub instead of raising RuntimeError when not tracing (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.take_snapshot: take_snapshot_without_start_raises (errors)."""
import tracemalloc

_raised = False
try:
    tracemalloc.take_snapshot()
except RuntimeError:
    _raised = True
assert _raised, "take_snapshot_without_start_raises: expected RuntimeError"
print("take_snapshot_without_start_raises OK")
"###);
    assert_output(&out, r###"take_snapshot_without_start_raises OK
"###);
}

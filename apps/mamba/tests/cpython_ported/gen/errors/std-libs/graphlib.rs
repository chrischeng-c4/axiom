use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/graphlib/done_before_prepare_raises.py`.
#[test]
fn test_gen_errors_std_libs_graphlib_done_before_prepare_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "errors"
# case = "done_before_prepare_raises"
# subject = "graphlib.TopologicalSorter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: done_before_prepare_raises (errors)."""
import graphlib

_raised = False
try:
    graphlib.TopologicalSorter().done(3)
except ValueError:
    _raised = True
assert _raised, "done_before_prepare_raises: expected ValueError"
print("done_before_prepare_raises OK")
"###);
    assert_output(&out, r###"done_before_prepare_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/graphlib/done_not_passed_out_raises.py`.
#[test]
fn test_gen_errors_std_libs_graphlib_done_not_passed_out_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "errors"
# case = "done_not_passed_out_raises"
# subject = "graphlib.TopologicalSorter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: done_not_passed_out_raises (errors)."""
import graphlib

_raised = False
try:
    (lambda t: (t.add(1, 2, 3, 4), t.add(2, 3, 4), t.prepare(), t.get_ready(), t.done(2)))(graphlib.TopologicalSorter())
except ValueError:
    _raised = True
assert _raised, "done_not_passed_out_raises: expected ValueError"
print("done_not_passed_out_raises OK")
"###);
    assert_output(&out, r###"done_not_passed_out_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/graphlib/done_unknown_node_raises.py`.
#[test]
fn test_gen_errors_std_libs_graphlib_done_unknown_node_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "errors"
# case = "done_unknown_node_raises"
# subject = "graphlib.TopologicalSorter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: done_unknown_node_raises (errors)."""
import graphlib

_raised = False
try:
    (lambda t: (t.add(1, 2), t.prepare(), t.done(24)))(graphlib.TopologicalSorter())
except ValueError:
    _raised = True
assert _raised, "done_unknown_node_raises: expected ValueError"
print("done_unknown_node_raises OK")
"###);
    assert_output(&out, r###"done_unknown_node_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/graphlib/get_ready_before_prepare_raises.py`.
#[test]
fn test_gen_errors_std_libs_graphlib_get_ready_before_prepare_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "errors"
# case = "get_ready_before_prepare_raises"
# subject = "graphlib.TopologicalSorter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: get_ready_before_prepare_raises (errors)."""
import graphlib

_raised = False
try:
    graphlib.TopologicalSorter().get_ready()
except ValueError:
    _raised = True
assert _raised, "get_ready_before_prepare_raises: expected ValueError"
print("get_ready_before_prepare_raises OK")
"###);
    assert_output(&out, r###"get_ready_before_prepare_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/graphlib/is_active_before_prepare_raises.py`.
#[test]
fn test_gen_errors_std_libs_graphlib_is_active_before_prepare_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "errors"
# case = "is_active_before_prepare_raises"
# subject = "graphlib.TopologicalSorter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: is_active_before_prepare_raises (errors)."""
import graphlib

_raised = False
try:
    graphlib.TopologicalSorter().is_active()
except ValueError:
    _raised = True
assert _raised, "is_active_before_prepare_raises: expected ValueError"
print("is_active_before_prepare_raises OK")
"###);
    assert_output(&out, r###"is_active_before_prepare_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/graphlib/long_cycle_raises.py`.
#[test]
fn test_gen_errors_std_libs_graphlib_long_cycle_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "errors"
# case = "long_cycle_raises"
# subject = "graphlib.TopologicalSorter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: long_cycle_raises (errors)."""
import graphlib

_raised = False
try:
    list(graphlib.TopologicalSorter({'X': {'Y'}, 'Y': {'Z'}, 'Z': {'X'}}).static_order())
except graphlib.CycleError:
    _raised = True
assert _raised, "long_cycle_raises: expected graphlib.CycleError"
print("long_cycle_raises OK")
"###);
    assert_output(&out, r###"long_cycle_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/graphlib/prepare_twice_raises.py`.
#[test]
fn test_gen_errors_std_libs_graphlib_prepare_twice_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "errors"
# case = "prepare_twice_raises"
# subject = "graphlib.TopologicalSorter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: prepare_twice_raises (errors)."""
import graphlib

_raised = False
try:
    (lambda t: (t.prepare(), t.prepare()))(graphlib.TopologicalSorter())
except ValueError:
    _raised = True
assert _raised, "prepare_twice_raises: expected ValueError"
print("prepare_twice_raises OK")
"###);
    assert_output(&out, r###"prepare_twice_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/graphlib/simple_cycle_raises.py`.
#[test]
fn test_gen_errors_std_libs_graphlib_simple_cycle_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "errors"
# case = "simple_cycle_raises"
# subject = "graphlib.TopologicalSorter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: simple_cycle_raises (errors)."""
import graphlib

_raised = False
try:
    list(graphlib.TopologicalSorter({'A': {'B'}, 'B': {'A'}}).static_order())
except graphlib.CycleError:
    _raised = True
assert _raised, "simple_cycle_raises: expected graphlib.CycleError"
print("simple_cycle_raises OK")
"###);
    assert_output(&out, r###"simple_cycle_raises OK
"###);
}

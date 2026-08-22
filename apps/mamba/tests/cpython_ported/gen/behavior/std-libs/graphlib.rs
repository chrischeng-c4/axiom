use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/graphlib/generator_deps_consumed.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_generator_deps_consumed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "generator_deps_consumed"
# subject = "graphlib.TopologicalSorter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: dependencies may be any iterable including a one-shot generator: {0: (2*x+1 for x in range(5))} gives static_order [1,3,5,7,9,0]"""
import graphlib

deps = (2 * x + 1 for x in range(5))
gen_ts = graphlib.TopologicalSorter({0: deps})
gen_order = list(gen_ts.static_order())
assert gen_order == [1, 3, 5, 7, 9, 0], gen_order

print("generator_deps_consumed OK")
"###);
    assert_output(&out, r###"generator_deps_consumed OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/graphlib/incremental_add_equals_batched.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_incremental_add_equals_batched() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "incremental_add_equals_batched"
# subject = "graphlib.TopologicalSorter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: repeated add(1,dep) calls accumulate to the same static_order as a single add of the dep set {2,3,4,5}"""
import graphlib

incremental = graphlib.TopologicalSorter()
incremental.add(1, 2)
incremental.add(1, 3)
incremental.add(1, 4)
incremental.add(1, 5)
batched = graphlib.TopologicalSorter({1: {2, 3, 4, 5}})
incremental_order = list(incremental.static_order())
assert incremental_order == list(batched.static_order()), incremental_order

print("incremental_add_equals_batched OK")
"###);
    assert_output(&out, r###"incremental_add_equals_batched OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/graphlib/insertion_order_groups_invariant.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_insertion_order_groups_invariant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "insertion_order_groups_invariant"
# subject = "graphlib.TopologicalSorter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: the order edges are added in does not change the grouping produced by the parallel prepare/get_ready/done driver loop"""
import graphlib


def groups(ts):
    ts.prepare()
    out = []
    while ts.is_active():
        ready = ts.get_ready()
        ts.done(*ready)
        out.append(set(ready))
    return out


a = graphlib.TopologicalSorter()
a.add(3, 2, 1)
a.add(1, 0)
a.add(4, 5)
a.add(6, 7)
a.add(4, 7)

b = graphlib.TopologicalSorter()
b.add(1, 0)
b.add(3, 2, 1)
b.add(4, 7)
b.add(6, 7)
b.add(4, 5)

assert groups(a) == groups(b), "insertion order does not change groups"

print("insertion_order_groups_invariant OK")
"###);
    assert_output(&out, r###"insertion_order_groups_invariant OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/graphlib/manual_driver_loop_lifecycle.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_manual_driver_loop_lifecycle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "manual_driver_loop_lifecycle"
# subject = "graphlib.TopologicalSorter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: prepare/get_ready/done/is_active drive parallel scheduling: leaves come out first as a tuple, done() unblocks dependents, is_active stays True until every released node is done"""
import graphlib

# Node 1 depends on 2, 3, 4; node 2 also depends on 3.
ts = graphlib.TopologicalSorter()
ts.add(1, 2, 3, 4)
ts.add(2, 3)
ts.prepare()

# Leaves with no outstanding deps come out first, as a tuple.
first = ts.get_ready()
assert set(first) == {3, 4}, sorted(first)

# Nothing new is ready until we mark progress.
assert ts.get_ready() == (), "no new nodes before done"

# Completing 3 unblocks 2 (4 still outstanding for 1).
ts.done(3)
assert ts.get_ready() == (2,), "ready after done(3)"
assert ts.get_ready() == (), "drained again"

# Finish 4 and 2; that unblocks 1.
ts.done(4)
ts.done(2)
assert ts.get_ready() == (1,), "ready after done(4) and done(2)"
assert ts.get_ready() == (), "drained again"

# is_active stays True until every released node is marked done.
assert ts.is_active() is True, "still active before final done"
ts.done(1)
assert ts.get_ready() == (), "nothing left"
assert ts.is_active() is False, "inactive once fully drained"

print("manual_driver_loop_lifecycle OK")
"###);
    assert_output(&out, r###"manual_driver_loop_lifecycle OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/graphlib/noarg_construction_succeeds.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_noarg_construction_succeeds() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "noarg_construction_succeeds"
# subject = "graphlib.TopologicalSorter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""graphlib.TopologicalSorter: TopologicalSorter is callable and a no-arg construction succeeds, yielding a non-None object on both runtimes"""
import graphlib

assert callable(graphlib.TopologicalSorter), "TopologicalSorter must be callable"
ts = graphlib.TopologicalSorter()
assert ts is not None, "no-arg construction yields an object"

print("noarg_construction_succeeds OK")
"###);
    assert_output(&out, r###"noarg_construction_succeeds OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/graphlib/static_order_linear_chain.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_static_order_linear_chain() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "static_order_linear_chain"
# subject = "graphlib.TopologicalSorter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: static_order on the chain A<-B<-C ({'A':{'B'},'B':{'C'},'C':set()}) yields dependencies before dependents: ['C','B','A']"""
import graphlib

ts = graphlib.TopologicalSorter({"A": {"B"}, "B": {"C"}, "C": set()})
order = list(ts.static_order())
assert order == ["C", "B", "A"], order

print("static_order_linear_chain OK")
"###);
    assert_output(&out, r###"static_order_linear_chain OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/graphlib/test_topological_sort__test_add_dependencies_for_same_node_incrementally.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_test_topological_sort__test_add_dependencies_for_same_node_incrementally() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_add_dependencies_for_same_node_incrementally"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_add_dependencies_for_same_node_incrementally"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_add_dependencies_for_same_node_incrementally
"""Auto-ported test: TestTopologicalSort::test_add_dependencies_for_same_node_incrementally (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
ts = graphlib.TopologicalSorter()
ts.add(1, 2)
ts.add(1, 3)
ts.add(1, 4)
ts.add(1, 5)
ts2 = graphlib.TopologicalSorter({1: {2, 3, 4, 5}})

assert [*ts.static_order()] == [*ts2.static_order()]
print("TestTopologicalSort::test_add_dependencies_for_same_node_incrementally: ok")
"###);
    assert_output(&out, r###"TestTopologicalSort::test_add_dependencies_for_same_node_incrementally: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/graphlib/test_topological_sort__test_calls_before_prepare.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_test_topological_sort__test_calls_before_prepare() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_calls_before_prepare"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_calls_before_prepare"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_calls_before_prepare
"""Auto-ported test: TestTopologicalSort::test_calls_before_prepare (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
ts = graphlib.TopologicalSorter()
try:
    ts.get_ready()
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('prepare\\(\\) must be called first', str(_aR_e))
try:
    ts.done(3)
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('prepare\\(\\) must be called first', str(_aR_e))
try:
    ts.is_active()
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('prepare\\(\\) must be called first', str(_aR_e))
print("TestTopologicalSort::test_calls_before_prepare: ok")
"###);
    assert_output(&out, r###"TestTopologicalSort::test_calls_before_prepare: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/graphlib/test_topological_sort__test_done.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_test_topological_sort__test_done() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_done"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_done"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_done
"""Auto-ported test: TestTopologicalSort::test_done (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
ts = graphlib.TopologicalSorter()
ts.add(1, 2, 3, 4)
ts.add(2, 3)
ts.prepare()

assert ts.get_ready() == (3, 4)

assert ts.get_ready() == ()
ts.done(3)

assert ts.get_ready() == (2,)

assert ts.get_ready() == ()
ts.done(4)
ts.done(2)

assert ts.get_ready() == (1,)

assert ts.get_ready() == ()
ts.done(1)

assert ts.get_ready() == ()

assert not ts.is_active()
print("TestTopologicalSort::test_done: ok")
"###);
    assert_output(&out, r###"TestTopologicalSort::test_done: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/graphlib/test_topological_sort__test_empty.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_test_topological_sort__test_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_empty"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_empty"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_empty
"""Auto-ported test: TestTopologicalSort::test_empty (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
def _assert_cycle(graph, cycle):
    ts = graphlib.TopologicalSorter()
    for node, dependson in graph.items():
        ts.add(node, *dependson)
    try:
        ts.prepare()
    except graphlib.CycleError as e:
        _, seq = e.args

        assert ' '.join(map(str, cycle)) in ' '.join(map(str, seq * 2))
    else:
        raise

def _test_graph(graph, expected):

    def static_order_with_groups(ts):
        ts.prepare()
        while ts.is_active():
            nodes = ts.get_ready()
            for node in nodes:
                ts.done(node)
            yield tuple(sorted(nodes))
    ts = graphlib.TopologicalSorter(graph)

    assert list(static_order_with_groups(ts)) == list(expected)
    ts = graphlib.TopologicalSorter(graph)
    it = iter(ts.static_order())
    for group in expected:
        tsgroup = {next(it) for element in group}

        assert set(group) == tsgroup
_test_graph({}, [])
print("TestTopologicalSort::test_empty: ok")
"###);
    assert_output(&out, r###"TestTopologicalSort::test_empty: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/graphlib/test_topological_sort__test_graph_with_iterables.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_test_topological_sort__test_graph_with_iterables() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_graph_with_iterables"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_graph_with_iterables"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_graph_with_iterables
"""Auto-ported test: TestTopologicalSort::test_graph_with_iterables (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
dependson = (2 * x + 1 for x in range(5))
ts = graphlib.TopologicalSorter({0: dependson})

assert list(ts.static_order()) == [1, 3, 5, 7, 9, 0]
print("TestTopologicalSort::test_graph_with_iterables: ok")
"###);
    assert_output(&out, r###"TestTopologicalSort::test_graph_with_iterables: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/graphlib/test_topological_sort__test_invalid_nodes_in_done.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_test_topological_sort__test_invalid_nodes_in_done() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_invalid_nodes_in_done"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_invalid_nodes_in_done"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_invalid_nodes_in_done
"""Auto-ported test: TestTopologicalSort::test_invalid_nodes_in_done (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
ts = graphlib.TopologicalSorter()
ts.add(1, 2, 3, 4)
ts.add(2, 3, 4)
ts.prepare()
ts.get_ready()
try:
    ts.done(2)
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('node 2 was not passed out', str(_aR_e))
try:
    ts.done(24)
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('node 24 was not added using add\\(\\)', str(_aR_e))
print("TestTopologicalSort::test_invalid_nodes_in_done: ok")
"###);
    assert_output(&out, r###"TestTopologicalSort::test_invalid_nodes_in_done: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/graphlib/test_topological_sort__test_is_active.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_test_topological_sort__test_is_active() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_is_active"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_is_active"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_is_active
"""Auto-ported test: TestTopologicalSort::test_is_active (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
ts = graphlib.TopologicalSorter()
ts.add(1, 2)
ts.prepare()

assert ts.is_active()

assert ts.get_ready() == (2,)

assert ts.is_active()
ts.done(2)

assert ts.is_active()

assert ts.get_ready() == (1,)

assert ts.is_active()
ts.done(1)

assert not ts.is_active()
print("TestTopologicalSort::test_is_active: ok")
"###);
    assert_output(&out, r###"TestTopologicalSort::test_is_active: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/graphlib/test_topological_sort__test_no_dependencies.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_test_topological_sort__test_no_dependencies() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_no_dependencies"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_no_dependencies"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_no_dependencies
"""Auto-ported test: TestTopologicalSort::test_no_dependencies (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
def _assert_cycle(graph, cycle):
    ts = graphlib.TopologicalSorter()
    for node, dependson in graph.items():
        ts.add(node, *dependson)
    try:
        ts.prepare()
    except graphlib.CycleError as e:
        _, seq = e.args

        assert ' '.join(map(str, cycle)) in ' '.join(map(str, seq * 2))
    else:
        raise

def _test_graph(graph, expected):

    def static_order_with_groups(ts):
        ts.prepare()
        while ts.is_active():
            nodes = ts.get_ready()
            for node in nodes:
                ts.done(node)
            yield tuple(sorted(nodes))
    ts = graphlib.TopologicalSorter(graph)

    assert list(static_order_with_groups(ts)) == list(expected)
    ts = graphlib.TopologicalSorter(graph)
    it = iter(ts.static_order())
    for group in expected:
        tsgroup = {next(it) for element in group}

        assert set(group) == tsgroup
_test_graph({1: {2}, 3: {4}, 5: {6}}, [(2, 4, 6), (1, 3, 5)])
_test_graph({1: set(), 3: set(), 5: set()}, [(1, 3, 5)])
print("TestTopologicalSort::test_no_dependencies: ok")
"###);
    assert_output(&out, r###"TestTopologicalSort::test_no_dependencies: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/graphlib/test_topological_sort__test_prepare_multiple_times.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_test_topological_sort__test_prepare_multiple_times() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_prepare_multiple_times"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_prepare_multiple_times"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_prepare_multiple_times
"""Auto-ported test: TestTopologicalSort::test_prepare_multiple_times (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
ts = graphlib.TopologicalSorter()
ts.prepare()
try:
    ts.prepare()
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('cannot prepare\\(\\) more than once', str(_aR_e))
print("TestTopologicalSort::test_prepare_multiple_times: ok")
"###);
    assert_output(&out, r###"TestTopologicalSort::test_prepare_multiple_times: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/graphlib/test_topological_sort__test_simple_cases.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_test_topological_sort__test_simple_cases() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_simple_cases"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_simple_cases"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_simple_cases
"""Auto-ported test: TestTopologicalSort::test_simple_cases (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
def _assert_cycle(graph, cycle):
    ts = graphlib.TopologicalSorter()
    for node, dependson in graph.items():
        ts.add(node, *dependson)
    try:
        ts.prepare()
    except graphlib.CycleError as e:
        _, seq = e.args

        assert ' '.join(map(str, cycle)) in ' '.join(map(str, seq * 2))
    else:
        raise

def _test_graph(graph, expected):

    def static_order_with_groups(ts):
        ts.prepare()
        while ts.is_active():
            nodes = ts.get_ready()
            for node in nodes:
                ts.done(node)
            yield tuple(sorted(nodes))
    ts = graphlib.TopologicalSorter(graph)

    assert list(static_order_with_groups(ts)) == list(expected)
    ts = graphlib.TopologicalSorter(graph)
    it = iter(ts.static_order())
    for group in expected:
        tsgroup = {next(it) for element in group}

        assert set(group) == tsgroup
_test_graph({2: {11}, 9: {11, 8}, 10: {11, 3}, 11: {7, 5}, 8: {7, 3}}, [(3, 5, 7), (8, 11), (2, 9, 10)])
_test_graph({1: {}}, [(1,)])
_test_graph({x: {x + 1} for x in range(10)}, [(x,) for x in range(10, -1, -1)])
_test_graph({2: {3}, 3: {4}, 4: {5}, 5: {1}, 11: {12}, 12: {13}, 13: {14}, 14: {15}}, [(1, 15), (5, 14), (4, 13), (3, 12), (2, 11)])
_test_graph({0: [1, 2], 1: [3], 2: [5, 6], 3: [4], 4: [9], 5: [3], 6: [7], 7: [8], 8: [4], 9: []}, [(9,), (4,), (3, 8), (1, 5, 7), (6,), (2,), (0,)])
_test_graph({0: [1, 2], 1: [], 2: [3], 3: []}, [(1, 3), (2,), (0,)])
_test_graph({0: [1, 2], 1: [], 2: [3], 3: [], 4: [5], 5: [6], 6: []}, [(1, 3, 6), (2, 5), (0, 4)])
print("TestTopologicalSort::test_simple_cases: ok")
"###);
    assert_output(&out, r###"TestTopologicalSort::test_simple_cases: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/graphlib/test_topological_sort__test_static_order_does_not_change_with_the_hash_seed.py`.
#[test]
fn test_gen_behavior_std_libs_graphlib_test_topological_sort__test_static_order_does_not_change_with_the_hash_seed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_static_order_does_not_change_with_the_hash_seed"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_static_order_does_not_change_with_the_hash_seed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_static_order_does_not_change_with_the_hash_seed
"""Auto-ported test: TestTopologicalSort::test_static_order_does_not_change_with_the_hash_seed (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
def check_order_with_hash_seed(seed):
    code = "if 1:\n                import graphlib\n                ts = graphlib.TopologicalSorter()\n                ts.add('blech', 'bluch', 'hola')\n                ts.add('abcd', 'blech', 'bluch', 'a', 'b')\n                ts.add('a', 'a string', 'something', 'b')\n                ts.add('bluch', 'hola', 'abcde', 'a', 'b')\n                print(list(ts.static_order()))\n                "
    env = os.environ.copy()
    env['__cleanenv'] = True
    env['PYTHONHASHSEED'] = str(seed)
    out = assert_python_ok('-c', code, **env)
    return out
run1 = check_order_with_hash_seed(1234)
run2 = check_order_with_hash_seed(31415)

assert run1 != ''

assert run2 != ''

assert run1 == run2
print("TestTopologicalSort::test_static_order_does_not_change_with_the_hash_seed: ok")
"###);
    assert_output(&out, r###"TestTopologicalSort::test_static_order_does_not_change_with_the_hash_seed: ok
"###);
}

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_cycle"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_cycle"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_cycle
"""Auto-ported test: TestTopologicalSort::test_cycle (CPython 3.12 oracle)."""


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
_assert_cycle({1: {1}}, [1, 1])
_assert_cycle({1: {2}, 2: {1}}, [1, 2, 1])
_assert_cycle({1: {2}, 2: {3}, 3: {1}}, [1, 3, 2, 1])
_assert_cycle({1: {2}, 2: {3}, 3: {1}, 5: {4}, 4: {6}}, [1, 3, 2, 1])
_assert_cycle({1: {2}, 2: {1}, 3: {4}, 4: {5}, 6: {7}, 7: {6}}, [1, 2, 1])
_assert_cycle({1: {2}, 2: {3}, 3: {2, 4}, 4: {5}}, [3, 2])
print("TestTopologicalSort::test_cycle: ok")

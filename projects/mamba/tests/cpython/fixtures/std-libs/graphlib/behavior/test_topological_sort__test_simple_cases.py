# /// script
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
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
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

use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/heapq/heapify_maintains_invariant.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_heapify_maintains_invariant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heapify_maintains_invariant"
# subject = "heapq.heapify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heapify: after heapify every parent heap[i] is <= its children heap[2i+1] and heap[2i+2] (the min-heap invariant)"""
import heapq

_lst = [9, 4, 7, 2, 5]
heapq.heapify(_lst)
assert _lst[0] == 2, f"heapify min = {_lst[0]!r}"
# Heap invariant: for all i, heap[i] <= heap[2*i+1] and heap[2*i+2].
for _i in range(len(_lst)):
    if 2 * _i + 1 < len(_lst):
        assert _lst[_i] <= _lst[2 * _i + 1], f"heap invariant left child at {_i}"
    if 2 * _i + 2 < len(_lst):
        assert _lst[_i] <= _lst[2 * _i + 2], f"heap invariant right child at {_i}"
print("heapify_maintains_invariant OK")
"###);
    assert_output(&out, r###"heapify_maintains_invariant OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/heapify_min_at_root.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_heapify_min_at_root() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heapify_min_at_root"
# subject = "heapq.heapify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heapify: heapify rearranges a list in place so the minimum element sits at index 0 (the root)"""
import heapq

_lst = [10, 5, 3, 8, 1]
heapq.heapify(_lst)
assert _lst[0] == min([10, 5, 3, 8, 1]), f"heapify min at root = {_lst[0]!r}"
print("heapify_min_at_root OK")
"###);
    assert_output(&out, r###"heapify_min_at_root OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/heappop_yields_ascending.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_heappop_yields_ascending() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heappop_yields_ascending"
# subject = "heapq.heappop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappop: draining a heapified list via repeated heappop yields the elements in ascending (sorted) order"""
import heapq

_h = [5, 3, 7, 1, 9, 2]
heapq.heapify(_h)
_popped = []
while _h:
    _popped.append(heapq.heappop(_h))
assert _popped == sorted([5, 3, 7, 1, 9, 2]), f"heap sort order = {_popped!r}"
print("heappop_yields_ascending OK")
"###);
    assert_output(&out, r###"heappop_yields_ascending OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/heappushpop_larger_pops_min.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_heappushpop_larger_pops_min() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heappushpop_larger_pops_min"
# subject = "heapq.heappushpop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappushpop: heappushpop with a value larger than the current min pops and returns the old min, and the new value is inserted into the heap"""
import heapq

_h5 = [1, 3, 5]
heapq.heapify(_h5)
_r5 = heapq.heappushpop(_h5, 10)
assert _r5 == 1, f"heappushpop(10 > 1) pops 1 = {_r5!r}"
assert 10 in _h5, "10 was inserted"
print("heappushpop_larger_pops_min OK")
"###);
    assert_output(&out, r###"heappushpop_larger_pops_min OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/heappushpop_single_element_larger.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_heappushpop_single_element_larger() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heappushpop_single_element_larger"
# subject = "heapq.heappushpop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappushpop: heappushpop on a one-element heap with a larger value returns the old min and the heap now holds only the new value"""
import heapq

_one = [10]
assert heapq.heappushpop(_one, 11) == 10, "pushpop larger returns old min"
assert _one == [11], f"heap now holds new value = {_one!r}"
print("heappushpop_single_element_larger OK")
"###);
    assert_output(&out, r###"heappushpop_single_element_larger OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/heappushpop_smaller_returns_input.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_heappushpop_smaller_returns_input() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heappushpop_smaller_returns_input"
# subject = "heapq.heappushpop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappushpop: heappushpop with a value smaller than the current min returns that value immediately and leaves the heap unchanged"""
import heapq

_h4 = [5, 6, 7]
heapq.heapify(_h4)
_r4 = heapq.heappushpop(_h4, 2)
assert _r4 == 2, f"heappushpop(2 < 5) = {_r4!r}"
assert _h4[0] == 5, f"heap unchanged min = {_h4[0]!r}"
print("heappushpop_smaller_returns_input OK")
"###);
    assert_output(&out, r###"heappushpop_smaller_returns_input OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/heapreplace_pops_then_pushes.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_heapreplace_pops_then_pushes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heapreplace_pops_then_pushes"
# subject = "heapq.heapreplace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heapreplace: heapreplace returns the current smallest and then inserts the new value; replacing the root of [1,2,3] with 0 returns 1 and leaves 0 at the root"""
import heapq

_h3 = [1, 2, 3]
heapq.heapify(_h3)
_old3 = heapq.heapreplace(_h3, 0)
assert _old3 == 1, f"heapreplace old = {_old3!r}"
# After replacing with 0, the heap min (root) is 0.
assert _h3[0] == 0, f"heapreplace new root = {_h3[0]!r}"
print("heapreplace_pops_then_pushes OK")
"###);
    assert_output(&out, r###"heapreplace_pops_then_pushes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/heapsort_property_both_paths.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_heapsort_property_both_paths() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heapsort_property_both_paths"
# subject = "heapq.heapify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heapify: draining a heap yields the same order as sorted() for both heap-construction paths (bulk heapify and incremental heappush), and nlargest/nsmallest agree with the sorted slices, over a deterministic pseudo-random sequence across several sizes"""
import heapq


def gen(n, seed=12345):
    """Deterministic LCG -> values in [0, 1000); no random module."""
    out = []
    state = seed
    for _ in range(n):
        state = (1103515245 * state + 12345) & 0x7FFFFFFF
        out.append(state % 1000)
    return out


for size in (0, 1, 2, 7, 50, 200):
    data = gen(size, seed=size + 1)
    expected = sorted(data)

    # Path A: bulk heapify, then drain.
    heap_a = data[:]
    heapq.heapify(heap_a)
    drained_a = [heapq.heappop(heap_a) for _ in range(size)]
    assert drained_a == expected, f"heapify-drain size={size}"
    assert heap_a == [], f"heap fully drained size={size}"

    # Path B: incremental heappush, then drain.
    heap_b = []
    for item in data:
        heapq.heappush(heap_b, item)
    drained_b = [heapq.heappop(heap_b) for _ in range(size)]
    assert drained_b == expected, f"heappush-drain size={size}"

    # nlargest/nsmallest agree with the sorted slices.
    for k in (0, 1, 3, size, size + 5):
        assert heapq.nlargest(k, data) == expected[::-1][:k], (
            f"nlargest k={k} size={size}"
        )
        assert heapq.nsmallest(k, data) == expected[:k], (
            f"nsmallest k={k} size={size}"
        )
print("heapsort_property_both_paths OK")
"###);
    assert_output(&out, r###"heapsort_property_both_paths OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/merge_empty_inputs.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_merge_empty_inputs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "merge_empty_inputs"
# subject = "heapq.merge"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.merge: merge() of empty inputs (and of no inputs, and with key=) yields nothing"""
import heapq

assert list(heapq.merge([], [])) == [], "merge of empties is empty"
assert list(heapq.merge([], [], key=len)) == [], "merge of empties with key="
assert list(heapq.merge()) == [], "merge of no inputs is empty"
print("merge_empty_inputs OK")
"###);
    assert_output(&out, r###"merge_empty_inputs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/merge_is_stable_on_ties.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_merge_is_stable_on_ties() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "merge_is_stable_on_ties"
# subject = "heapq.merge"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.merge: merge() is stable: records with equal keys keep their original per-stream input order within each key band"""
import heapq

# Tag each record with (key, source) so we can detect reordering of ties.
streams = [
    [(0, "a0"), (0, "a1"), (1, "a2")],
    [(0, "b0"), (1, "b1"), (1, "b2")],
    [(0, "c0"), (2, "c1")],
]
stable = list(heapq.merge(*streams, key=lambda r: r[0]))
# All key==0 records must precede key==1, which precede key==2, and within
# each key band the original per-stream order is preserved.
keys_only = [k for k, _ in stable]
assert keys_only == sorted(keys_only), f"merge not sorted by key = {keys_only!r}"
zero_band = [tag for k, tag in stable if k == 0]
assert zero_band == ["a0", "a1", "b0", "c0"], f"tie order lost = {zero_band!r}"
print("merge_is_stable_on_ties OK")
"###);
    assert_output(&out, r###"merge_is_stable_on_ties OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/merge_key_projects_order.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_merge_key_projects_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "merge_key_projects_order"
# subject = "heapq.merge"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.merge: merge(key=) merges on the projected key order, not the raw element order, interleaving two row streams by their numeric field"""
import heapq

rows_a = [("A", 1), ("B", 4), ("C", 7)]
rows_b = [("D", 2), ("E", 5), ("F", 8)]
merged_by_num = list(heapq.merge(rows_a, rows_b, key=lambda r: r[1]))
assert merged_by_num == [
    ("A", 1), ("D", 2), ("B", 4), ("E", 5), ("C", 7), ("F", 8)
], f"merge key= = {merged_by_num!r}"
print("merge_key_projects_order OK")
"###);
    assert_output(&out, r###"merge_key_projects_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/merge_kway_multi_input.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_merge_kway_multi_input() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "merge_kway_multi_input"
# subject = "heapq.merge"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.merge: merge() combines more than two sorted inputs k-way in a single pass"""
import heapq

multi = list(heapq.merge([1, 5], [2, 6], [3, 7], [4, 8]))
assert multi == [1, 2, 3, 4, 5, 6, 7, 8], f"k-way merge = {multi!r}"
print("merge_kway_multi_input OK")
"###);
    assert_output(&out, r###"merge_kway_multi_input OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/merge_propagates_input_exceptions.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_merge_propagates_input_exceptions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "merge_propagates_input_exceptions"
# subject = "heapq.merge"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.merge: merge() does not swallow exceptions raised by its input iterators; an IndexError from a faulty generator surfaces to the caller"""
import heapq


# This generator over-indexes its backing list and raises IndexError, which
# must surface to the caller instead of being silently dropped.
def faulty():
    backing = list(range(5))
    for i in range(10):  # i reaches 5..9 -> IndexError
        yield backing[i]


raised = False
try:
    list(heapq.merge(faulty(), faulty()))
except IndexError:
    raised = True
assert raised, "merge must propagate IndexError from inputs"
print("merge_propagates_input_exceptions OK")
"###);
    assert_output(&out, r###"merge_propagates_input_exceptions OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/merge_reverse_descending.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_merge_reverse_descending() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "merge_reverse_descending"
# subject = "heapq.merge"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.merge: merge(reverse=True) merges descending inputs into a single descending result"""
import heapq

desc_a = [9, 6, 3]
desc_b = [8, 5, 2]
merged_desc = list(heapq.merge(desc_a, desc_b, reverse=True))
assert merged_desc == [9, 8, 6, 5, 3, 2], f"merge reverse= = {merged_desc!r}"
print("merge_reverse_descending OK")
"###);
    assert_output(&out, r###"merge_reverse_descending OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/merge_two_sorted_inputs.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_merge_two_sorted_inputs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "merge_two_sorted_inputs"
# subject = "heapq.merge"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.merge: merge() lazily combines two already-sorted inputs into one ascending iterator"""
import heapq

_m = list(heapq.merge([1, 3, 5], [2, 4, 6]))
assert _m == [1, 2, 3, 4, 5, 6], f"merge = {_m!r}"
print("merge_two_sorted_inputs OK")
"###);
    assert_output(&out, r###"merge_two_sorted_inputs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/nlargest_nsmallest_clamp_n.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_nlargest_nsmallest_clamp_n() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "nlargest_nsmallest_clamp_n"
# subject = "heapq.nlargest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.nlargest: nlargest/nsmallest clamp n at the boundaries: n=0 returns [] and n>len returns the whole sequence fully sorted"""
import heapq

_seq = [4, 1, 7, 3, 9, 2]
assert heapq.nlargest(0, _seq) == [], "nlargest(0) is empty"
assert heapq.nsmallest(0, _seq) == [], "nsmallest(0) is empty"
assert heapq.nlargest(100, _seq) == sorted(_seq, reverse=True), "nlargest(n>len)"
assert heapq.nsmallest(100, _seq) == sorted(_seq), "nsmallest(n>len)"
print("nlargest_nsmallest_clamp_n OK")
"###);
    assert_output(&out, r###"nlargest_nsmallest_clamp_n OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/nlargest_with_key.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_nlargest_with_key() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "nlargest_with_key"
# subject = "heapq.nlargest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.nlargest: nlargest(n, data, key=len) selects the n elements with the largest key, ordered largest-key first"""
import heapq

_data = ["banana", "apple", "cherry", "date", "elderberry"]
_top2 = heapq.nlargest(2, _data, key=len)
assert len(_top2[0]) >= len(_top2[1]), f"nlargest by len = {_top2!r}"
assert _top2[0] == "elderberry", f"longest = {_top2[0]!r}"
print("nlargest_with_key OK")
"###);
    assert_output(&out, r###"nlargest_with_key OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/nsmallest_with_key.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_nsmallest_with_key() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "nsmallest_with_key"
# subject = "heapq.nsmallest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.nsmallest: nsmallest(n, data, key=...) selects the n elements with the smallest projected key, smallest first"""
import heapq

_nums = [(-3, "a"), (1, "b"), (-1, "c"), (2, "d")]
_small2 = heapq.nsmallest(2, _nums, key=lambda x: x[0])
assert _small2[0][0] == -3, f"nsmallest by key = {_small2!r}"
print("nsmallest_with_key OK")
"###);
    assert_output(&out, r###"nsmallest_with_key OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_error_handling_c__test_arg_parsing.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_error_handling_c__test_arg_parsing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_error_handling_c__test_arg_parsing"
# subject = "cpython.test_heapq.TestErrorHandlingC.test_arg_parsing"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestErrorHandlingC::test_arg_parsing
"""Auto-ported test: TestErrorHandlingC::test_arg_parsing (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = c_heapq
for f in (module.heapify, module.heappop, module.heappush, module.heapreplace, module.nlargest, module.nsmallest):

    try:
        f(10)
        raise AssertionError('expected (TypeError, AttributeError)')
    except (TypeError, AttributeError):
        pass
print("TestErrorHandlingC::test_arg_parsing: ok")
"###);
    assert_output(&out, r###"TestErrorHandlingC::test_arg_parsing: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_error_handling_c__test_len_only.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_error_handling_c__test_len_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_error_handling_c__test_len_only"
# subject = "cpython.test_heapq.TestErrorHandlingC.test_len_only"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestErrorHandlingC::test_len_only
"""Auto-ported test: TestErrorHandlingC::test_len_only (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = c_heapq
for f in (module.heapify, module.heappop):

    try:
        f(LenOnly())
        raise AssertionError('expected (TypeError, AttributeError)')
    except (TypeError, AttributeError):
        pass
for f in (module.heappush, module.heapreplace):

    try:
        f(LenOnly(), 10)
        raise AssertionError('expected (TypeError, AttributeError)')
    except (TypeError, AttributeError):
        pass
for f in (module.nlargest, module.nsmallest):

    try:
        f(2, LenOnly())
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("TestErrorHandlingC::test_len_only: ok")
"###);
    assert_output(&out, r###"TestErrorHandlingC::test_len_only: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_error_handling_c__test_non_sequence.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_error_handling_c__test_non_sequence() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_error_handling_c__test_non_sequence"
# subject = "cpython.test_heapq.TestErrorHandlingC.test_non_sequence"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestErrorHandlingC::test_non_sequence
"""Auto-ported test: TestErrorHandlingC::test_non_sequence (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = c_heapq
for f in (module.heapify, module.heappop):

    try:
        f(10)
        raise AssertionError('expected (TypeError, AttributeError)')
    except (TypeError, AttributeError):
        pass
for f in (module.heappush, module.heapreplace, module.nlargest, module.nsmallest):

    try:
        f(10, 10)
        raise AssertionError('expected (TypeError, AttributeError)')
    except (TypeError, AttributeError):
        pass
print("TestErrorHandlingC::test_non_sequence: ok")
"###);
    assert_output(&out, r###"TestErrorHandlingC::test_non_sequence: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_error_handling_python__test_arg_parsing.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_error_handling_python__test_arg_parsing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_error_handling_python__test_arg_parsing"
# subject = "cpython.test_heapq.TestErrorHandlingPython.test_arg_parsing"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestErrorHandlingPython::test_arg_parsing
"""Auto-ported test: TestErrorHandlingPython::test_arg_parsing (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = py_heapq
for f in (module.heapify, module.heappop, module.heappush, module.heapreplace, module.nlargest, module.nsmallest):

    try:
        f(10)
        raise AssertionError('expected (TypeError, AttributeError)')
    except (TypeError, AttributeError):
        pass
print("TestErrorHandlingPython::test_arg_parsing: ok")
"###);
    assert_output(&out, r###"TestErrorHandlingPython::test_arg_parsing: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_error_handling_python__test_len_only.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_error_handling_python__test_len_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_error_handling_python__test_len_only"
# subject = "cpython.test_heapq.TestErrorHandlingPython.test_len_only"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestErrorHandlingPython::test_len_only
"""Auto-ported test: TestErrorHandlingPython::test_len_only (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = py_heapq
for f in (module.heapify, module.heappop):

    try:
        f(LenOnly())
        raise AssertionError('expected (TypeError, AttributeError)')
    except (TypeError, AttributeError):
        pass
for f in (module.heappush, module.heapreplace):

    try:
        f(LenOnly(), 10)
        raise AssertionError('expected (TypeError, AttributeError)')
    except (TypeError, AttributeError):
        pass
for f in (module.nlargest, module.nsmallest):

    try:
        f(2, LenOnly())
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("TestErrorHandlingPython::test_len_only: ok")
"###);
    assert_output(&out, r###"TestErrorHandlingPython::test_len_only: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_error_handling_python__test_non_sequence.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_error_handling_python__test_non_sequence() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_error_handling_python__test_non_sequence"
# subject = "cpython.test_heapq.TestErrorHandlingPython.test_non_sequence"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestErrorHandlingPython::test_non_sequence
"""Auto-ported test: TestErrorHandlingPython::test_non_sequence (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = py_heapq
for f in (module.heapify, module.heappop):

    try:
        f(10)
        raise AssertionError('expected (TypeError, AttributeError)')
    except (TypeError, AttributeError):
        pass
for f in (module.heappush, module.heapreplace, module.nlargest, module.nsmallest):

    try:
        f(10, 10)
        raise AssertionError('expected (TypeError, AttributeError)')
    except (TypeError, AttributeError):
        pass
print("TestErrorHandlingPython::test_non_sequence: ok")
"###);
    assert_output(&out, r###"TestErrorHandlingPython::test_non_sequence: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_c__test_empty_merges.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_c__test_empty_merges() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_c__test_empty_merges"
# subject = "cpython.test_heapq.TestHeapC.test_empty_merges"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapC::test_empty_merges
"""Auto-ported test: TestHeapC::test_empty_merges (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = c_heapq

assert list(module.merge([], [])) == []

assert list(module.merge([], [], key=lambda: 6)) == []
print("TestHeapC::test_empty_merges: ok")
"###);
    assert_output(&out, r###"TestHeapC::test_empty_merges: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_c__test_heapify.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_c__test_heapify() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_c__test_heapify"
# subject = "cpython.test_heapq.TestHeapC.test_heapify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapC::test_heapify
"""Auto-ported test: TestHeapC::test_heapify (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = c_heapq

def check_invariant(heap):
    for pos, item in enumerate(heap):
        if pos:
            parentpos = pos - 1 >> 1

            assert heap[parentpos] <= item

def heapiter(heap):
    try:
        while 1:
            yield module.heappop(heap)
    except IndexError:
        pass
for size in list(range(30)) + [20000]:
    heap = [random.random() for dummy in range(size)]
    module.heapify(heap)
    check_invariant(heap)

try:
    module.heapify(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestHeapC::test_heapify: ok")
"###);
    assert_output(&out, r###"TestHeapC::test_heapify: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_c__test_heappop_max.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_c__test_heappop_max() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_c__test_heappop_max"
# subject = "cpython.test_heapq.TestHeapC.test_heappop_max"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapC::test_heappop_max
"""Auto-ported test: TestHeapC::test_heappop_max (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = c_heapq
h = [3, 2]

assert module._heappop_max(h) == 3

assert module._heappop_max(h) == 2
print("TestHeapC::test_heappop_max: ok")
"###);
    assert_output(&out, r###"TestHeapC::test_heappop_max: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_c__test_heappushpop.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_c__test_heappushpop() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_c__test_heappushpop"
# subject = "cpython.test_heapq.TestHeapC.test_heappushpop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapC::test_heappushpop
"""Auto-ported test: TestHeapC::test_heappushpop (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = c_heapq
h = []
x = module.heappushpop(h, 10)

assert (h, x) == ([], 10)
h = [10]
x = module.heappushpop(h, 10.0)

assert (h, x) == ([10], 10.0)

assert type(h[0]) == int

assert type(x) == float
h = [10]
x = module.heappushpop(h, 9)

assert (h, x) == ([10], 9)
h = [10]
x = module.heappushpop(h, 11)

assert (h, x) == ([11], 10)
print("TestHeapC::test_heappushpop: ok")
"###);
    assert_output(&out, r###"TestHeapC::test_heappushpop: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_c__test_heapsort.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_c__test_heapsort() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_c__test_heapsort"
# subject = "cpython.test_heapq.TestHeapC.test_heapsort"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapC::test_heapsort
"""Auto-ported test: TestHeapC::test_heapsort (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = c_heapq
for trial in range(100):
    size = random.randrange(50)
    data = [random.randrange(25) for i in range(size)]
    if trial & 1:
        heap = data[:]
        module.heapify(heap)
    else:
        heap = []
        for item in data:
            module.heappush(heap, item)
    heap_sorted = [module.heappop(heap) for i in range(size)]

    assert heap_sorted == sorted(data)
print("TestHeapC::test_heapsort: ok")
"###);
    assert_output(&out, r###"TestHeapC::test_heapsort: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_c__test_naive_nbest.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_c__test_naive_nbest() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_c__test_naive_nbest"
# subject = "cpython.test_heapq.TestHeapC.test_naive_nbest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapC::test_naive_nbest
"""Auto-ported test: TestHeapC::test_naive_nbest (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = c_heapq
data = [random.randrange(2000) for i in range(1000)]
heap = []
for item in data:
    module.heappush(heap, item)
    if len(heap) > 10:
        module.heappop(heap)
heap.sort()

assert heap == sorted(data)[-10:]
print("TestHeapC::test_naive_nbest: ok")
"###);
    assert_output(&out, r###"TestHeapC::test_naive_nbest: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_c__test_nbest.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_c__test_nbest() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_c__test_nbest"
# subject = "cpython.test_heapq.TestHeapC.test_nbest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapC::test_nbest
"""Auto-ported test: TestHeapC::test_nbest (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = c_heapq

def check_invariant(heap):
    for pos, item in enumerate(heap):
        if pos:
            parentpos = pos - 1 >> 1

            assert heap[parentpos] <= item

def heapiter(heap):
    try:
        while 1:
            yield module.heappop(heap)
    except IndexError:
        pass
data = [random.randrange(2000) for i in range(1000)]
heap = data[:10]
module.heapify(heap)
for item in data[10:]:
    if item > heap[0]:
        module.heapreplace(heap, item)

assert list(heapiter(heap)) == sorted(data)[-10:]

try:
    module.heapreplace(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    module.heapreplace(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    module.heapreplace([], None)
    raise AssertionError('expected IndexError')
except IndexError:
    pass
print("TestHeapC::test_nbest: ok")
"###);
    assert_output(&out, r###"TestHeapC::test_nbest: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_c__test_nbest_with_pushpop.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_c__test_nbest_with_pushpop() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_c__test_nbest_with_pushpop"
# subject = "cpython.test_heapq.TestHeapC.test_nbest_with_pushpop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapC::test_nbest_with_pushpop
"""Auto-ported test: TestHeapC::test_nbest_with_pushpop (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = c_heapq

def check_invariant(heap):
    for pos, item in enumerate(heap):
        if pos:
            parentpos = pos - 1 >> 1

            assert heap[parentpos] <= item

def heapiter(heap):
    try:
        while 1:
            yield module.heappop(heap)
    except IndexError:
        pass
data = [random.randrange(2000) for i in range(1000)]
heap = data[:10]
module.heapify(heap)
for item in data[10:]:
    module.heappushpop(heap, item)

assert list(heapiter(heap)) == sorted(data)[-10:]

assert module.heappushpop([], 'x') == 'x'
print("TestHeapC::test_nbest_with_pushpop: ok")
"###);
    assert_output(&out, r###"TestHeapC::test_nbest_with_pushpop: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_c__test_push_pop.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_c__test_push_pop() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_c__test_push_pop"
# subject = "cpython.test_heapq.TestHeapC.test_push_pop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapC::test_push_pop
"""Auto-ported test: TestHeapC::test_push_pop (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = c_heapq

def check_invariant(heap):
    for pos, item in enumerate(heap):
        if pos:
            parentpos = pos - 1 >> 1

            assert heap[parentpos] <= item

def heapiter(heap):
    try:
        while 1:
            yield module.heappop(heap)
    except IndexError:
        pass
heap = []
data = []
check_invariant(heap)
for i in range(256):
    item = random.random()
    data.append(item)
    module.heappush(heap, item)
    check_invariant(heap)
results = []
while heap:
    item = module.heappop(heap)
    check_invariant(heap)
    results.append(item)
data_sorted = data[:]
data_sorted.sort()

assert data_sorted == results
check_invariant(results)

try:
    module.heappush([])
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:

    try:
        module.heappush(None, None)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        module.heappop(None)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
except AttributeError:
    pass
print("TestHeapC::test_push_pop: ok")
"###);
    assert_output(&out, r###"TestHeapC::test_push_pop: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_python__test_empty_merges.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_python__test_empty_merges() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_python__test_empty_merges"
# subject = "cpython.test_heapq.TestHeapPython.test_empty_merges"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapPython::test_empty_merges
"""Auto-ported test: TestHeapPython::test_empty_merges (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = py_heapq

assert list(module.merge([], [])) == []

assert list(module.merge([], [], key=lambda: 6)) == []
print("TestHeapPython::test_empty_merges: ok")
"###);
    assert_output(&out, r###"TestHeapPython::test_empty_merges: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_python__test_heapify.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_python__test_heapify() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_python__test_heapify"
# subject = "cpython.test_heapq.TestHeapPython.test_heapify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapPython::test_heapify
"""Auto-ported test: TestHeapPython::test_heapify (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = py_heapq

def check_invariant(heap):
    for pos, item in enumerate(heap):
        if pos:
            parentpos = pos - 1 >> 1

            assert heap[parentpos] <= item

def heapiter(heap):
    try:
        while 1:
            yield module.heappop(heap)
    except IndexError:
        pass
for size in list(range(30)) + [20000]:
    heap = [random.random() for dummy in range(size)]
    module.heapify(heap)
    check_invariant(heap)

try:
    module.heapify(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestHeapPython::test_heapify: ok")
"###);
    assert_output(&out, r###"TestHeapPython::test_heapify: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_python__test_heappop_max.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_python__test_heappop_max() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_python__test_heappop_max"
# subject = "cpython.test_heapq.TestHeapPython.test_heappop_max"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapPython::test_heappop_max
"""Auto-ported test: TestHeapPython::test_heappop_max (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = py_heapq
h = [3, 2]

assert module._heappop_max(h) == 3

assert module._heappop_max(h) == 2
print("TestHeapPython::test_heappop_max: ok")
"###);
    assert_output(&out, r###"TestHeapPython::test_heappop_max: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_python__test_heappushpop.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_python__test_heappushpop() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_python__test_heappushpop"
# subject = "cpython.test_heapq.TestHeapPython.test_heappushpop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapPython::test_heappushpop
"""Auto-ported test: TestHeapPython::test_heappushpop (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = py_heapq
h = []
x = module.heappushpop(h, 10)

assert (h, x) == ([], 10)
h = [10]
x = module.heappushpop(h, 10.0)

assert (h, x) == ([10], 10.0)

assert type(h[0]) == int

assert type(x) == float
h = [10]
x = module.heappushpop(h, 9)

assert (h, x) == ([10], 9)
h = [10]
x = module.heappushpop(h, 11)

assert (h, x) == ([11], 10)
print("TestHeapPython::test_heappushpop: ok")
"###);
    assert_output(&out, r###"TestHeapPython::test_heappushpop: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_python__test_heapsort.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_python__test_heapsort() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_python__test_heapsort"
# subject = "cpython.test_heapq.TestHeapPython.test_heapsort"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapPython::test_heapsort
"""Auto-ported test: TestHeapPython::test_heapsort (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = py_heapq
for trial in range(100):
    size = random.randrange(50)
    data = [random.randrange(25) for i in range(size)]
    if trial & 1:
        heap = data[:]
        module.heapify(heap)
    else:
        heap = []
        for item in data:
            module.heappush(heap, item)
    heap_sorted = [module.heappop(heap) for i in range(size)]

    assert heap_sorted == sorted(data)
print("TestHeapPython::test_heapsort: ok")
"###);
    assert_output(&out, r###"TestHeapPython::test_heapsort: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_python__test_naive_nbest.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_python__test_naive_nbest() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_python__test_naive_nbest"
# subject = "cpython.test_heapq.TestHeapPython.test_naive_nbest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapPython::test_naive_nbest
"""Auto-ported test: TestHeapPython::test_naive_nbest (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = py_heapq
data = [random.randrange(2000) for i in range(1000)]
heap = []
for item in data:
    module.heappush(heap, item)
    if len(heap) > 10:
        module.heappop(heap)
heap.sort()

assert heap == sorted(data)[-10:]
print("TestHeapPython::test_naive_nbest: ok")
"###);
    assert_output(&out, r###"TestHeapPython::test_naive_nbest: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_python__test_nbest.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_python__test_nbest() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_python__test_nbest"
# subject = "cpython.test_heapq.TestHeapPython.test_nbest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapPython::test_nbest
"""Auto-ported test: TestHeapPython::test_nbest (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = py_heapq

def check_invariant(heap):
    for pos, item in enumerate(heap):
        if pos:
            parentpos = pos - 1 >> 1

            assert heap[parentpos] <= item

def heapiter(heap):
    try:
        while 1:
            yield module.heappop(heap)
    except IndexError:
        pass
data = [random.randrange(2000) for i in range(1000)]
heap = data[:10]
module.heapify(heap)
for item in data[10:]:
    if item > heap[0]:
        module.heapreplace(heap, item)

assert list(heapiter(heap)) == sorted(data)[-10:]

try:
    module.heapreplace(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    module.heapreplace(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    module.heapreplace([], None)
    raise AssertionError('expected IndexError')
except IndexError:
    pass
print("TestHeapPython::test_nbest: ok")
"###);
    assert_output(&out, r###"TestHeapPython::test_nbest: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_python__test_nbest_with_pushpop.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_python__test_nbest_with_pushpop() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_python__test_nbest_with_pushpop"
# subject = "cpython.test_heapq.TestHeapPython.test_nbest_with_pushpop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapPython::test_nbest_with_pushpop
"""Auto-ported test: TestHeapPython::test_nbest_with_pushpop (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = py_heapq

def check_invariant(heap):
    for pos, item in enumerate(heap):
        if pos:
            parentpos = pos - 1 >> 1

            assert heap[parentpos] <= item

def heapiter(heap):
    try:
        while 1:
            yield module.heappop(heap)
    except IndexError:
        pass
data = [random.randrange(2000) for i in range(1000)]
heap = data[:10]
module.heapify(heap)
for item in data[10:]:
    module.heappushpop(heap, item)

assert list(heapiter(heap)) == sorted(data)[-10:]

assert module.heappushpop([], 'x') == 'x'
print("TestHeapPython::test_nbest_with_pushpop: ok")
"###);
    assert_output(&out, r###"TestHeapPython::test_nbest_with_pushpop: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/test_heap_python__test_push_pop.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_test_heap_python__test_push_pop() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_heap_python__test_push_pop"
# subject = "cpython.test_heapq.TestHeapPython.test_push_pop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestHeapPython::test_push_pop
"""Auto-ported test: TestHeapPython::test_push_pop (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = py_heapq

def check_invariant(heap):
    for pos, item in enumerate(heap):
        if pos:
            parentpos = pos - 1 >> 1

            assert heap[parentpos] <= item

def heapiter(heap):
    try:
        while 1:
            yield module.heappop(heap)
    except IndexError:
        pass
heap = []
data = []
check_invariant(heap)
for i in range(256):
    item = random.random()
    data.append(item)
    module.heappush(heap, item)
    check_invariant(heap)
results = []
while heap:
    item = module.heappop(heap)
    check_invariant(heap)
    results.append(item)
data_sorted = data[:]
data_sorted.sort()

assert data_sorted == results
check_invariant(results)

try:
    module.heappush([])
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:

    try:
        module.heappush(None, None)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        module.heappop(None)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
except AttributeError:
    pass
print("TestHeapPython::test_push_pop: ok")
"###);
    assert_output(&out, r###"TestHeapPython::test_push_pop: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/heapq/tuples_ordered_lexicographically.py`.
#[test]
fn test_gen_behavior_std_libs_heapq_tuples_ordered_lexicographically() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "tuples_ordered_lexicographically"
# subject = "heapq.heappush"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappush: tuples pushed onto a heap are compared lexicographically; popping yields them in ascending tuple order"""
import heapq

_th = []
heapq.heappush(_th, (2, "b"))
heapq.heappush(_th, (1, "a"))
heapq.heappush(_th, (3, "c"))
assert heapq.heappop(_th) == (1, "a"), "tuple heap smallest first"
assert heapq.heappop(_th) == (2, "b"), "tuple heap second"
print("tuples_ordered_lexicographically OK")
"###);
    assert_output(&out, r###"tuples_ordered_lexicographically OK
"###);
}

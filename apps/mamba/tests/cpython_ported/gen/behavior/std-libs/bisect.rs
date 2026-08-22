use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/bisect/absent_element_insertion_point.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_absent_element_insertion_point() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "absent_element_insertion_point"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: for an absent element both bisect_left and bisect_right give the same insertion index"""
import bisect

b = [10, 20, 30, 40]
assert bisect.bisect_left(b, 25) == 2, f"absent elem left = {bisect.bisect_left(b, 25)!r}"
assert bisect.bisect_right(b, 25) == 2, f"absent elem right = {bisect.bisect_right(b, 25)!r}"

print("absent_element_insertion_point OK")
"###);
    assert_output(&out, r###"absent_element_insertion_point OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/accepts_keyword_arguments.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_accepts_keyword_arguments() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "accepts_keyword_arguments"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
"""bisect.bisect_left: all functions accept a / x / lo / hi as keyword arguments"""
import bisect

data = [10, 20, 30, 40, 50]
assert bisect.bisect_left(a=data, x=25, lo=1, hi=3) == 2
assert bisect.bisect_right(a=data, x=25, lo=1, hi=3) == 2
assert bisect.bisect(a=data, x=25, lo=1, hi=3) == 2
bisect.insort_left(a=data, x=25, lo=1, hi=3)
bisect.insort_right(a=data, x=25, lo=1, hi=3)
bisect.insort(a=data, x=25, lo=1, hi=3)
assert data == [10, 20, 25, 25, 25, 30, 40, 50], f"keyword insort = {data!r}"

print("accepts_keyword_arguments OK")
"###);
    assert_output(&out, r###"accepts_keyword_arguments OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/bisect_alias_equals_bisect_right.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_bisect_alias_equals_bisect_right() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "bisect_alias_equals_bisect_right"
# subject = "bisect.bisect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect: bisect is an alias for bisect_right and returns the identical index"""
import bisect

lst = [1, 2, 4, 4, 5, 7]
assert bisect.bisect(lst, 4) == bisect.bisect_right(lst, 4), "bisect == bisect_right"

print("bisect_alias_equals_bisect_right OK")
"###);
    assert_output(&out, r###"bisect_alias_equals_bisect_right OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/bisect_left_leftmost_of_equal.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_bisect_left_leftmost_of_equal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "bisect_left_leftmost_of_equal"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: bisect_left returns the insertion point left of all equal elements"""
import bisect

a = [1, 2, 2, 2, 5]
assert bisect.bisect_left(a, 2) == 1, f"bisect_left leftmost = {bisect.bisect_left(a, 2)!r}"

print("bisect_left_leftmost_of_equal OK")
"###);
    assert_output(&out, r###"bisect_left_leftmost_of_equal OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/bisect_right_rightmost_of_equal.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_bisect_right_rightmost_of_equal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "bisect_right_rightmost_of_equal"
# subject = "bisect.bisect_right"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_right: bisect_right returns the insertion point right of all equal elements"""
import bisect

a = [1, 2, 2, 2, 5]
assert bisect.bisect_right(a, 2) == 4, f"bisect_right rightmost = {bisect.bisect_right(a, 2)!r}"

print("bisect_right_rightmost_of_equal OK")
"###);
    assert_output(&out, r###"bisect_right_rightmost_of_equal OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/empty_list_returns_zero.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_empty_list_returns_zero() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "empty_list_returns_zero"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: bisect_left and bisect_right on an empty list both return 0"""
import bisect

assert bisect.bisect_left([], 5) == 0, "empty list bisect_left = 0"
assert bisect.bisect_right([], 5) == 0, "empty list bisect_right = 0"

print("empty_list_returns_zero OK")
"###);
    assert_output(&out, r###"empty_list_returns_zero OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/grade_lookup_via_bisect.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_grade_lookup_via_bisect() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "grade_lookup_via_bisect"
# subject = "bisect.bisect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect: the classic grade-band lookup: bisect maps a score to its grade bucket"""
import bisect

grades = [60, 70, 80, 90]
marks = ['F', 'D', 'C', 'B', 'A']

def grade(score):
    return marks[bisect.bisect(grades, score)]

assert grade(55) == 'F', f"grade 55 = {grade(55)!r}"
assert grade(75) == 'C', f"grade 75 = {grade(75)!r}"
assert grade(95) == 'A', f"grade 95 = {grade(95)!r}"

print("grade_lookup_via_bisect OK")
"###);
    assert_output(&out, r###"grade_lookup_via_bisect OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/insort_alias_equals_insort_right.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_insort_alias_equals_insort_right() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "insort_alias_equals_insort_right"
# subject = "bisect.insort"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.insort: insort is an alias for insort_right and produces the identical list"""
import bisect

via_insort = [1, 2, 3]
bisect.insort(via_insort, 2)
via_right = [1, 2, 3]
bisect.insort_right(via_right, 2)
assert via_insort == via_right, f"insort == insort_right ({via_insort!r} vs {via_right!r})"

print("insort_alias_equals_insort_right OK")
"###);
    assert_output(&out, r###"insort_alias_equals_insort_right OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/insort_builds_sorted_from_scratch.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_insort_builds_sorted_from_scratch() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "insort_builds_sorted_from_scratch"
# subject = "bisect.insort"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.insort: repeated insort on an initially-empty list builds a fully sorted list"""
import bisect

f = []
for v in [5, 3, 1, 4, 2]:
    bisect.insort(f, v)
assert f == [1, 2, 3, 4, 5], f"insort sort = {f!r}"

print("insort_builds_sorted_from_scratch OK")
"###);
    assert_output(&out, r###"insort_builds_sorted_from_scratch OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/insort_calls_sequence_insert.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_insort_calls_sequence_insert() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "insort_calls_sequence_insert"
# subject = "bisect.insort_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
"""bisect.insort_left: insort uses the sequence's own insert(); a list subclass overriding insert observes the call"""
import bisect


# insort calls the sequence's own insert(): a list subclass that overrides
# insert observes the insertion (here it redirects into .store).
class TrackingList(list):
    def __init__(self):
        super().__init__()
        self.store = []

    def insert(self, index, item):
        self.store.insert(index, item)


lst = TrackingList()
bisect.insort_left(lst, 10)
bisect.insort_right(lst, 5)
assert lst.store == [5, 10], f"list-subclass insert = {lst.store!r}"

print("insort_calls_sequence_insert OK")
"###);
    assert_output(&out, r###"insort_calls_sequence_insert OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/insort_honors_key.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_insort_honors_key() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "insort_honors_key"
# subject = "bisect.insort_right"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
"""bisect.insort_right: insort_right honors key= and keeps the list sorted by the projected key"""
import bisect

rows = [("a", 1), ("b", 3), ("c", 5)]
bisect.insort_right(rows, ("x", 4), key=lambda r: r[1])
assert rows == [("a", 1), ("b", 3), ("x", 4), ("c", 5)], f"insort key= = {rows!r}"

print("insort_honors_key OK")
"###);
    assert_output(&out, r###"insort_honors_key OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/insort_left_inserts_left_of_equal.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_insort_left_inserts_left_of_equal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "insort_left_inserts_left_of_equal"
# subject = "bisect.insort_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.insort_left: insort_left keeps the list sorted and inserts left of an equal run"""
import bisect

c = [1, 3, 3, 5]
bisect.insort_left(c, 3)
assert c == [1, 3, 3, 3, 5], f"insort_left sorted = {c!r}"

print("insort_left_inserts_left_of_equal OK")
"###);
    assert_output(&out, r###"insort_left_inserts_left_of_equal OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/insort_right_inserts_right_of_equal.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_insort_right_inserts_right_of_equal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "insort_right_inserts_right_of_equal"
# subject = "bisect.insort_right"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.insort_right: insort_right keeps the list sorted and inserts right of an equal run"""
import bisect

d = [1, 3, 3, 5]
bisect.insort_right(d, 3)
assert d == [1, 3, 3, 3, 5], f"insort_right sorted = {d!r}"

print("insort_right_inserts_right_of_equal OK")
"###);
    assert_output(&out, r###"insort_right_inserts_right_of_equal OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/key_equals_precomputed_search.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_key_equals_precomputed_search() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "key_equals_precomputed_search"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
"""bisect.bisect_left: bisect_*(arr, x, key=f) matches searching the precomputed [f(v) for v in arr] with x"""
import bisect

# key= search over a list sorted by abs() equals searching the precomputed
# key list with the bare value.
keyfunc = abs
arr = sorted([2, -4, 6, 8, -10], key=keyfunc)   # [2, -4, 6, 8, -10]
precomputed = [keyfunc(v) for v in arr]         # [2, 4, 6, 8, 10]
for x in precomputed:
    assert bisect.bisect_left(arr, x, key=keyfunc) == bisect.bisect_left(precomputed, x)
    assert bisect.bisect_right(arr, x, key=keyfunc) == bisect.bisect_right(precomputed, x)

# Same equivalence with a string-casefold key over mixed-case letters.
kf = str.casefold
letters = sorted("aBcDeEfg", key=kf)
pre = [kf(v) for v in letters]
for x in pre:
    assert bisect.bisect_left(letters, x, key=kf) == bisect.bisect_left(pre, x)
    assert bisect.bisect_right(letters, x, key=kf) == bisect.bisect_right(pre, x)

print("key_equals_precomputed_search OK")
"###);
    assert_output(&out, r###"key_equals_precomputed_search OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/lo_hi_restrict_search_range.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_lo_hi_restrict_search_range() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "lo_hi_restrict_search_range"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: lo and hi confine the binary search to the [lo, hi) sub-window"""
import bisect

e = [0, 1, 2, 3, 4, 5]
# search restricted to [2, 5): element 3 maps to index 3
assert bisect.bisect_left(e, 3, lo=2, hi=5) == 3, f"lo/hi range = {bisect.bisect_left(e, 3, lo=2, hi=5)!r}"
# a value below the window clamps to lo
assert bisect.bisect_left(e, 1, lo=2, hi=4) == 2, "below-window clamps to lo"
# a value above the window clamps to hi
assert bisect.bisect_left(e, 9, lo=2, hi=4) == 4, "above-window clamps to hi"

print("lo_hi_restrict_search_range OK")
"###);
    assert_output(&out, r###"lo_hi_restrict_search_range OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/nonbool_lt_taken_by_truthiness.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_nonbool_lt_taken_by_truthiness() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "nonbool_lt_taken_by_truthiness"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
"""bisect.bisect_left: a __lt__ returning a non-bool is interpreted by truthiness during the search"""
import bisect


# __lt__ returns a non-bool (truthy/falsy str); bisect takes it by truthiness.
class NonBool:
    def __init__(self, val):
        self.val = val

    def __lt__(self, other):
        return "nonempty" if self.val < other.val else ""


data = [NonBool(i) for i in range(100)]
assert bisect.bisect_left(data, NonBool(33)) == 33, "non-bool __lt__ left"
assert bisect.bisect_right(data, NonBool(33)) == 34, "non-bool __lt__ right"

print("nonbool_lt_taken_by_truthiness OK")
"###);
    assert_output(&out, r###"nonbool_lt_taken_by_truthiness OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/notimplemented_lt_falls_back_to_gt.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_notimplemented_lt_falls_back_to_gt() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "notimplemented_lt_falls_back_to_gt"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
"""bisect.bisect_left: when __lt__ returns NotImplemented the reflected __gt__ drives the comparison"""
import bisect


# __lt__ returns NotImplemented -> Python falls back to the reflected __gt__.
class FallBack:
    def __init__(self, val):
        self.val = val

    def __lt__(self, other):
        return NotImplemented

    def __gt__(self, other):
        return self.val > other.val


d2 = [FallBack(i) for i in range(100)]
assert bisect.bisect_left(d2, FallBack(40)) == 40, "notimplemented fallback left"
assert bisect.bisect_right(d2, FallBack(40)) == 41, "notimplemented fallback right"

print("notimplemented_lt_falls_back_to_gt OK")
"###);
    assert_output(&out, r###"notimplemented_lt_falls_back_to_gt OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/return_type_is_int.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_return_type_is_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "return_type_is_int"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: bisect_left returns a plain int insertion index"""
import bisect

result = bisect.bisect_left([1, 2, 3], 2)
assert isinstance(result, int), f"return type is int, got {type(result).__name__}"
assert result == 1, f"bisect_left = {result!r}"

print("return_type_is_int OK")
"###);
    assert_output(&out, r###"return_type_is_int OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/string_sequence_bisect.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_string_sequence_bisect() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "string_sequence_bisect"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: bisect works on any comparable sorted sequence, e.g. a sorted list of strings"""
import bisect

words = ["apple", "cherry", "mango", "orange"]
pos = bisect.bisect_left(words, "grape")
assert pos == 2, f"string bisect = {pos!r}"

print("string_sequence_bisect OK")
"###);
    assert_output(&out, r###"string_sequence_bisect OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_c__test_backcompatibility.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_c__test_backcompatibility() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_c__test_backcompatibility"
# subject = "cpython.test_bisect.TestBisectC.test_backcompatibility"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectC::test_backcompatibility
"""Auto-ported test: TestBisectC::test_backcompatibility (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]

assert module.bisect == module.bisect_right
print("TestBisectC::test_backcompatibility: ok")
"###);
    assert_output(&out, r###"TestBisectC::test_backcompatibility: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_c__test_insort.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_c__test_insort() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_c__test_insort"
# subject = "cpython.test_bisect.TestBisectC.test_insort"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectC::test_insort
"""Auto-ported test: TestBisectC::test_insort (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
from random import shuffle
mod = module
keyfunc = abs
data = list(range(-10, 11)) + list(range(-20, 20, 2))
shuffle(data)
target = []
for x in data:
    mod.insort_left(target, x, key=keyfunc)

    assert sorted(target, key=keyfunc) == target
target = []
for x in data:
    mod.insort_right(target, x, key=keyfunc)

    assert sorted(target, key=keyfunc) == target
print("TestBisectC::test_insort: ok")
"###);
    assert_output(&out, r###"TestBisectC::test_insort: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_c__test_insort_keynot_none.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_c__test_insort_keynot_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_c__test_insort_keynot_none"
# subject = "cpython.test_bisect.TestBisectC.test_insort_keynotNone"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectC::test_insort_keynotNone
"""Auto-ported test: TestBisectC::test_insort_keynotNone (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
x = []
y = {'a': 2, 'b': 1}
for f in (module.insort_left, module.insort_right):

    try:
        f(x, y, key='b')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("TestBisectC::test_insort_keynotNone: ok")
"###);
    assert_output(&out, r###"TestBisectC::test_insort_keynotNone: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_c__test_keyword_args.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_c__test_keyword_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_c__test_keyword_args"
# subject = "cpython.test_bisect.TestBisectC.test_keyword_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectC::test_keyword_args
"""Auto-ported test: TestBisectC::test_keyword_args (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
data = [10, 20, 30, 40, 50]

assert module.bisect_left(a=data, x=25, lo=1, hi=3) == 2

assert module.bisect_right(a=data, x=25, lo=1, hi=3) == 2

assert module.bisect(a=data, x=25, lo=1, hi=3) == 2
module.insort_left(a=data, x=25, lo=1, hi=3)
module.insort_right(a=data, x=25, lo=1, hi=3)
module.insort(a=data, x=25, lo=1, hi=3)

assert data == [10, 20, 25, 25, 25, 30, 40, 50]
print("TestBisectC::test_keyword_args: ok")
"###);
    assert_output(&out, r###"TestBisectC::test_keyword_args: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_c__test_large_pyrange.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_c__test_large_pyrange() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_c__test_large_pyrange"
# subject = "cpython.test_bisect.TestBisectC.test_large_pyrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectC::test_large_pyrange
"""Auto-ported test: TestBisectC::test_large_pyrange (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
mod = module
n = sys.maxsize
data = Range(0, n - 1)

assert mod.bisect_left(data, n - 3) == n - 3

assert mod.bisect_right(data, n - 3) == n - 2

assert mod.bisect_left(data, n - 3, n - 10, n) == n - 3

assert mod.bisect_right(data, n - 3, n - 10, n) == n - 2
x = n - 100
mod.insort_left(data, x, x - 50, x + 50)

assert data.last_insert == (x, x)
x = n - 200
mod.insort_right(data, x, x - 50, x + 50)

assert data.last_insert == (x + 1, x)
print("TestBisectC::test_large_pyrange: ok")
"###);
    assert_output(&out, r###"TestBisectC::test_large_pyrange: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_c__test_large_range.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_c__test_large_range() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_c__test_large_range"
# subject = "cpython.test_bisect.TestBisectC.test_large_range"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectC::test_large_range
"""Auto-ported test: TestBisectC::test_large_range (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
mod = module
n = sys.maxsize
data = range(n - 1)

assert mod.bisect_left(data, n - 3) == n - 3

assert mod.bisect_right(data, n - 3) == n - 2

assert mod.bisect_left(data, n - 3, n - 10, n) == n - 3

assert mod.bisect_right(data, n - 3, n - 10, n) == n - 2
print("TestBisectC::test_large_range: ok")
"###);
    assert_output(&out, r###"TestBisectC::test_large_range: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_c__test_lookups_with_key_function.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_c__test_lookups_with_key_function() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_c__test_lookups_with_key_function"
# subject = "cpython.test_bisect.TestBisectC.test_lookups_with_key_function"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectC::test_lookups_with_key_function
"""Auto-ported test: TestBisectC::test_lookups_with_key_function (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
mod = module
keyfunc = abs
arr = sorted([2, -4, 6, 8, -10], key=keyfunc)
precomputed_arr = list(map(keyfunc, arr))
for x in precomputed_arr:

    assert mod.bisect_left(arr, x, key=keyfunc) == mod.bisect_left(precomputed_arr, x)

    assert mod.bisect_right(arr, x, key=keyfunc) == mod.bisect_right(precomputed_arr, x)
keyfunc = str.casefold
arr = sorted('aBcDeEfgHhiIiij', key=keyfunc)
precomputed_arr = list(map(keyfunc, arr))
for x in precomputed_arr:

    assert mod.bisect_left(arr, x, key=keyfunc) == mod.bisect_left(precomputed_arr, x)

    assert mod.bisect_right(arr, x, key=keyfunc) == mod.bisect_right(precomputed_arr, x)
print("TestBisectC::test_lookups_with_key_function: ok")
"###);
    assert_output(&out, r###"TestBisectC::test_lookups_with_key_function: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_c__test_lt_returns_non_bool.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_c__test_lt_returns_non_bool() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_c__test_lt_returns_non_bool"
# subject = "cpython.test_bisect.TestBisectC.test_lt_returns_non_bool"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectC::test_lt_returns_non_bool
"""Auto-ported test: TestBisectC::test_lt_returns_non_bool (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]

class A:

    def __init__(self, val):
        self.val = val

    def __lt__(self, other):
        return 'nonempty' if self.val < other.val else ''
data = [A(i) for i in range(100)]
i1 = module.bisect_left(data, A(33))
i2 = module.bisect_right(data, A(33))

assert i1 == 33

assert i2 == 34
print("TestBisectC::test_lt_returns_non_bool: ok")
"###);
    assert_output(&out, r###"TestBisectC::test_lt_returns_non_bool: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_c__test_lt_returns_notimplemented.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_c__test_lt_returns_notimplemented() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_c__test_lt_returns_notimplemented"
# subject = "cpython.test_bisect.TestBisectC.test_lt_returns_notimplemented"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectC::test_lt_returns_notimplemented
"""Auto-ported test: TestBisectC::test_lt_returns_notimplemented (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]

class A:

    def __init__(self, val):
        self.val = val

    def __lt__(self, other):
        return NotImplemented

    def __gt__(self, other):
        return self.val > other.val
data = [A(i) for i in range(100)]
i1 = module.bisect_left(data, A(40))
i2 = module.bisect_right(data, A(40))

assert i1 == 40

assert i2 == 41
print("TestBisectC::test_lt_returns_notimplemented: ok")
"###);
    assert_output(&out, r###"TestBisectC::test_lt_returns_notimplemented: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_c__test_negative_lo.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_c__test_negative_lo() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_c__test_negative_lo"
# subject = "cpython.test_bisect.TestBisectC.test_negative_lo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectC::test_negative_lo
"""Auto-ported test: TestBisectC::test_negative_lo (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
mod = module

try:
    mod.bisect_left([1, 2, 3], 5, -1, 3)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    mod.bisect_right([1, 2, 3], 5, -1, 3)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    mod.insort_left([1, 2, 3], 5, -1, 3)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    mod.insort_right([1, 2, 3], 5, -1, 3)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("TestBisectC::test_negative_lo: ok")
"###);
    assert_output(&out, r###"TestBisectC::test_negative_lo: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_c__test_optional_slicing.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_c__test_optional_slicing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_c__test_optional_slicing"
# subject = "cpython.test_bisect.TestBisectC.test_optionalSlicing"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectC::test_optionalSlicing
"""Auto-ported test: TestBisectC::test_optionalSlicing (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
for func, data, elem, expected in self_precomputedCases:
    for lo in range(4):
        lo = min(len(data), lo)
        for hi in range(3, 8):
            hi = min(len(data), hi)
            ip = func(data, elem, lo, hi)

            assert lo <= ip <= hi
            if func is module.bisect_left and ip < hi:

                assert elem <= data[ip]
            if func is module.bisect_left and ip > lo:

                assert data[ip - 1] < elem
            if func is module.bisect_right and ip < hi:

                assert elem < data[ip]
            if func is module.bisect_right and ip > lo:

                assert data[ip - 1] <= elem

            assert ip == max(lo, min(hi, expected))
print("TestBisectC::test_optionalSlicing: ok")
"###);
    assert_output(&out, r###"TestBisectC::test_optionalSlicing: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_c__test_precomputed.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_c__test_precomputed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_c__test_precomputed"
# subject = "cpython.test_bisect.TestBisectC.test_precomputed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectC::test_precomputed
"""Auto-ported test: TestBisectC::test_precomputed (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
for func, data, elem, expected in self_precomputedCases:

    assert func(data, elem) == expected

    assert func(UserList(data), elem) == expected
print("TestBisectC::test_precomputed: ok")
"###);
    assert_output(&out, r###"TestBisectC::test_precomputed: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_python__test_backcompatibility.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_python__test_backcompatibility() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_python__test_backcompatibility"
# subject = "cpython.test_bisect.TestBisectPython.test_backcompatibility"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectPython::test_backcompatibility
"""Auto-ported test: TestBisectPython::test_backcompatibility (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]

assert module.bisect == module.bisect_right
print("TestBisectPython::test_backcompatibility: ok")
"###);
    assert_output(&out, r###"TestBisectPython::test_backcompatibility: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_python__test_insort.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_python__test_insort() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_python__test_insort"
# subject = "cpython.test_bisect.TestBisectPython.test_insort"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectPython::test_insort
"""Auto-ported test: TestBisectPython::test_insort (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
from random import shuffle
mod = module
keyfunc = abs
data = list(range(-10, 11)) + list(range(-20, 20, 2))
shuffle(data)
target = []
for x in data:
    mod.insort_left(target, x, key=keyfunc)

    assert sorted(target, key=keyfunc) == target
target = []
for x in data:
    mod.insort_right(target, x, key=keyfunc)

    assert sorted(target, key=keyfunc) == target
print("TestBisectPython::test_insort: ok")
"###);
    assert_output(&out, r###"TestBisectPython::test_insort: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_python__test_insort_keynot_none.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_python__test_insort_keynot_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_python__test_insort_keynot_none"
# subject = "cpython.test_bisect.TestBisectPython.test_insort_keynotNone"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectPython::test_insort_keynotNone
"""Auto-ported test: TestBisectPython::test_insort_keynotNone (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
x = []
y = {'a': 2, 'b': 1}
for f in (module.insort_left, module.insort_right):

    try:
        f(x, y, key='b')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("TestBisectPython::test_insort_keynotNone: ok")
"###);
    assert_output(&out, r###"TestBisectPython::test_insort_keynotNone: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_python__test_keyword_args.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_python__test_keyword_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_python__test_keyword_args"
# subject = "cpython.test_bisect.TestBisectPython.test_keyword_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectPython::test_keyword_args
"""Auto-ported test: TestBisectPython::test_keyword_args (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
data = [10, 20, 30, 40, 50]

assert module.bisect_left(a=data, x=25, lo=1, hi=3) == 2

assert module.bisect_right(a=data, x=25, lo=1, hi=3) == 2

assert module.bisect(a=data, x=25, lo=1, hi=3) == 2
module.insort_left(a=data, x=25, lo=1, hi=3)
module.insort_right(a=data, x=25, lo=1, hi=3)
module.insort(a=data, x=25, lo=1, hi=3)

assert data == [10, 20, 25, 25, 25, 30, 40, 50]
print("TestBisectPython::test_keyword_args: ok")
"###);
    assert_output(&out, r###"TestBisectPython::test_keyword_args: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_python__test_large_pyrange.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_python__test_large_pyrange() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_python__test_large_pyrange"
# subject = "cpython.test_bisect.TestBisectPython.test_large_pyrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectPython::test_large_pyrange
"""Auto-ported test: TestBisectPython::test_large_pyrange (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
mod = module
n = sys.maxsize
data = Range(0, n - 1)

assert mod.bisect_left(data, n - 3) == n - 3

assert mod.bisect_right(data, n - 3) == n - 2

assert mod.bisect_left(data, n - 3, n - 10, n) == n - 3

assert mod.bisect_right(data, n - 3, n - 10, n) == n - 2
x = n - 100
mod.insort_left(data, x, x - 50, x + 50)

assert data.last_insert == (x, x)
x = n - 200
mod.insort_right(data, x, x - 50, x + 50)

assert data.last_insert == (x + 1, x)
print("TestBisectPython::test_large_pyrange: ok")
"###);
    assert_output(&out, r###"TestBisectPython::test_large_pyrange: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_python__test_large_range.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_python__test_large_range() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_python__test_large_range"
# subject = "cpython.test_bisect.TestBisectPython.test_large_range"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectPython::test_large_range
"""Auto-ported test: TestBisectPython::test_large_range (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
mod = module
n = sys.maxsize
data = range(n - 1)

assert mod.bisect_left(data, n - 3) == n - 3

assert mod.bisect_right(data, n - 3) == n - 2

assert mod.bisect_left(data, n - 3, n - 10, n) == n - 3

assert mod.bisect_right(data, n - 3, n - 10, n) == n - 2
print("TestBisectPython::test_large_range: ok")
"###);
    assert_output(&out, r###"TestBisectPython::test_large_range: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_python__test_lookups_with_key_function.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_python__test_lookups_with_key_function() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_python__test_lookups_with_key_function"
# subject = "cpython.test_bisect.TestBisectPython.test_lookups_with_key_function"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectPython::test_lookups_with_key_function
"""Auto-ported test: TestBisectPython::test_lookups_with_key_function (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
mod = module
keyfunc = abs
arr = sorted([2, -4, 6, 8, -10], key=keyfunc)
precomputed_arr = list(map(keyfunc, arr))
for x in precomputed_arr:

    assert mod.bisect_left(arr, x, key=keyfunc) == mod.bisect_left(precomputed_arr, x)

    assert mod.bisect_right(arr, x, key=keyfunc) == mod.bisect_right(precomputed_arr, x)
keyfunc = str.casefold
arr = sorted('aBcDeEfgHhiIiij', key=keyfunc)
precomputed_arr = list(map(keyfunc, arr))
for x in precomputed_arr:

    assert mod.bisect_left(arr, x, key=keyfunc) == mod.bisect_left(precomputed_arr, x)

    assert mod.bisect_right(arr, x, key=keyfunc) == mod.bisect_right(precomputed_arr, x)
print("TestBisectPython::test_lookups_with_key_function: ok")
"###);
    assert_output(&out, r###"TestBisectPython::test_lookups_with_key_function: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_python__test_lt_returns_non_bool.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_python__test_lt_returns_non_bool() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_python__test_lt_returns_non_bool"
# subject = "cpython.test_bisect.TestBisectPython.test_lt_returns_non_bool"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectPython::test_lt_returns_non_bool
"""Auto-ported test: TestBisectPython::test_lt_returns_non_bool (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]

class A:

    def __init__(self, val):
        self.val = val

    def __lt__(self, other):
        return 'nonempty' if self.val < other.val else ''
data = [A(i) for i in range(100)]
i1 = module.bisect_left(data, A(33))
i2 = module.bisect_right(data, A(33))

assert i1 == 33

assert i2 == 34
print("TestBisectPython::test_lt_returns_non_bool: ok")
"###);
    assert_output(&out, r###"TestBisectPython::test_lt_returns_non_bool: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_python__test_lt_returns_notimplemented.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_python__test_lt_returns_notimplemented() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_python__test_lt_returns_notimplemented"
# subject = "cpython.test_bisect.TestBisectPython.test_lt_returns_notimplemented"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectPython::test_lt_returns_notimplemented
"""Auto-ported test: TestBisectPython::test_lt_returns_notimplemented (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]

class A:

    def __init__(self, val):
        self.val = val

    def __lt__(self, other):
        return NotImplemented

    def __gt__(self, other):
        return self.val > other.val
data = [A(i) for i in range(100)]
i1 = module.bisect_left(data, A(40))
i2 = module.bisect_right(data, A(40))

assert i1 == 40

assert i2 == 41
print("TestBisectPython::test_lt_returns_notimplemented: ok")
"###);
    assert_output(&out, r###"TestBisectPython::test_lt_returns_notimplemented: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_python__test_negative_lo.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_python__test_negative_lo() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_python__test_negative_lo"
# subject = "cpython.test_bisect.TestBisectPython.test_negative_lo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectPython::test_negative_lo
"""Auto-ported test: TestBisectPython::test_negative_lo (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
mod = module

try:
    mod.bisect_left([1, 2, 3], 5, -1, 3)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    mod.bisect_right([1, 2, 3], 5, -1, 3)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    mod.insort_left([1, 2, 3], 5, -1, 3)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    mod.insort_right([1, 2, 3], 5, -1, 3)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("TestBisectPython::test_negative_lo: ok")
"###);
    assert_output(&out, r###"TestBisectPython::test_negative_lo: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_python__test_optional_slicing.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_python__test_optional_slicing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_python__test_optional_slicing"
# subject = "cpython.test_bisect.TestBisectPython.test_optionalSlicing"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectPython::test_optionalSlicing
"""Auto-ported test: TestBisectPython::test_optionalSlicing (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
for func, data, elem, expected in self_precomputedCases:
    for lo in range(4):
        lo = min(len(data), lo)
        for hi in range(3, 8):
            hi = min(len(data), hi)
            ip = func(data, elem, lo, hi)

            assert lo <= ip <= hi
            if func is module.bisect_left and ip < hi:

                assert elem <= data[ip]
            if func is module.bisect_left and ip > lo:

                assert data[ip - 1] < elem
            if func is module.bisect_right and ip < hi:

                assert elem < data[ip]
            if func is module.bisect_right and ip > lo:

                assert data[ip - 1] <= elem

            assert ip == max(lo, min(hi, expected))
print("TestBisectPython::test_optionalSlicing: ok")
"###);
    assert_output(&out, r###"TestBisectPython::test_optionalSlicing: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_bisect_python__test_precomputed.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_bisect_python__test_precomputed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_python__test_precomputed"
# subject = "cpython.test_bisect.TestBisectPython.test_precomputed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectPython::test_precomputed
"""Auto-ported test: TestBisectPython::test_precomputed (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
for func, data, elem, expected in self_precomputedCases:

    assert func(data, elem) == expected

    assert func(UserList(data), elem) == expected
print("TestBisectPython::test_precomputed: ok")
"###);
    assert_output(&out, r###"TestBisectPython::test_precomputed: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_doc_example_c__test_colors.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_doc_example_c__test_colors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_doc_example_c__test_colors"
# subject = "cpython.test_bisect.TestDocExampleC.test_colors"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestDocExampleC::test_colors
"""Auto-ported test: TestDocExampleC::test_colors (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
data = [('red', 5), ('blue', 1), ('yellow', 8), ('black', 0)]
data.sort(key=lambda r: r[1])
keys = [r[1] for r in data]
bisect_left = module.bisect_left

assert data[bisect_left(keys, 0)] == ('black', 0)

assert data[bisect_left(keys, 1)] == ('blue', 1)

assert data[bisect_left(keys, 5)] == ('red', 5)

assert data[bisect_left(keys, 8)] == ('yellow', 8)
print("TestDocExampleC::test_colors: ok")
"###);
    assert_output(&out, r###"TestDocExampleC::test_colors: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_doc_example_python__test_colors.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_doc_example_python__test_colors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_doc_example_python__test_colors"
# subject = "cpython.test_bisect.TestDocExamplePython.test_colors"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestDocExamplePython::test_colors
"""Auto-ported test: TestDocExamplePython::test_colors (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
data = [('red', 5), ('blue', 1), ('yellow', 8), ('black', 0)]
data.sort(key=lambda r: r[1])
keys = [r[1] for r in data]
bisect_left = module.bisect_left

assert data[bisect_left(keys, 0)] == ('black', 0)

assert data[bisect_left(keys, 1)] == ('blue', 1)

assert data[bisect_left(keys, 5)] == ('red', 5)

assert data[bisect_left(keys, 8)] == ('yellow', 8)
print("TestDocExamplePython::test_colors: ok")
"###);
    assert_output(&out, r###"TestDocExamplePython::test_colors: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_error_handling_c__test_arg_parsing.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_error_handling_c__test_arg_parsing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_error_handling_c__test_arg_parsing"
# subject = "cpython.test_bisect.TestErrorHandlingC.test_arg_parsing"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestErrorHandlingC::test_arg_parsing
"""Auto-ported test: TestErrorHandlingC::test_arg_parsing (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
for f in (module.bisect_left, module.bisect_right, module.insort_left, module.insort_right):

    try:
        f(10)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("TestErrorHandlingC::test_arg_parsing: ok")
"###);
    assert_output(&out, r###"TestErrorHandlingC::test_arg_parsing: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_error_handling_c__test_get_only.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_error_handling_c__test_get_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_error_handling_c__test_get_only"
# subject = "cpython.test_bisect.TestErrorHandlingC.test_get_only"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestErrorHandlingC::test_get_only
"""Auto-ported test: TestErrorHandlingC::test_get_only (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
for f in (module.bisect_left, module.bisect_right, module.insort_left, module.insort_right):

    try:
        f(GetOnly(), 10)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("TestErrorHandlingC::test_get_only: ok")
"###);
    assert_output(&out, r###"TestErrorHandlingC::test_get_only: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_error_handling_c__test_len_only.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_error_handling_c__test_len_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_error_handling_c__test_len_only"
# subject = "cpython.test_bisect.TestErrorHandlingC.test_len_only"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestErrorHandlingC::test_len_only
"""Auto-ported test: TestErrorHandlingC::test_len_only (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
for f in (module.bisect_left, module.bisect_right, module.insort_left, module.insort_right):

    try:
        f(LenOnly(), 10)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("TestErrorHandlingC::test_len_only: ok")
"###);
    assert_output(&out, r###"TestErrorHandlingC::test_len_only: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_error_handling_c__test_non_sequence.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_error_handling_c__test_non_sequence() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_error_handling_c__test_non_sequence"
# subject = "cpython.test_bisect.TestErrorHandlingC.test_non_sequence"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestErrorHandlingC::test_non_sequence
"""Auto-ported test: TestErrorHandlingC::test_non_sequence (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
for f in (module.bisect_left, module.bisect_right, module.insort_left, module.insort_right):

    try:
        f(10, 10)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("TestErrorHandlingC::test_non_sequence: ok")
"###);
    assert_output(&out, r###"TestErrorHandlingC::test_non_sequence: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_error_handling_python__test_arg_parsing.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_error_handling_python__test_arg_parsing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_error_handling_python__test_arg_parsing"
# subject = "cpython.test_bisect.TestErrorHandlingPython.test_arg_parsing"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestErrorHandlingPython::test_arg_parsing
"""Auto-ported test: TestErrorHandlingPython::test_arg_parsing (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
for f in (module.bisect_left, module.bisect_right, module.insort_left, module.insort_right):

    try:
        f(10)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("TestErrorHandlingPython::test_arg_parsing: ok")
"###);
    assert_output(&out, r###"TestErrorHandlingPython::test_arg_parsing: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_error_handling_python__test_get_only.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_error_handling_python__test_get_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_error_handling_python__test_get_only"
# subject = "cpython.test_bisect.TestErrorHandlingPython.test_get_only"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestErrorHandlingPython::test_get_only
"""Auto-ported test: TestErrorHandlingPython::test_get_only (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
for f in (module.bisect_left, module.bisect_right, module.insort_left, module.insort_right):

    try:
        f(GetOnly(), 10)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("TestErrorHandlingPython::test_get_only: ok")
"###);
    assert_output(&out, r###"TestErrorHandlingPython::test_get_only: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_error_handling_python__test_len_only.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_error_handling_python__test_len_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_error_handling_python__test_len_only"
# subject = "cpython.test_bisect.TestErrorHandlingPython.test_len_only"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestErrorHandlingPython::test_len_only
"""Auto-ported test: TestErrorHandlingPython::test_len_only (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
for f in (module.bisect_left, module.bisect_right, module.insort_left, module.insort_right):

    try:
        f(LenOnly(), 10)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("TestErrorHandlingPython::test_len_only: ok")
"###);
    assert_output(&out, r###"TestErrorHandlingPython::test_len_only: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_error_handling_python__test_non_sequence.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_error_handling_python__test_non_sequence() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_error_handling_python__test_non_sequence"
# subject = "cpython.test_bisect.TestErrorHandlingPython.test_non_sequence"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestErrorHandlingPython::test_non_sequence
"""Auto-ported test: TestErrorHandlingPython::test_non_sequence (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect
for f in (module.bisect_left, module.bisect_right, module.insort_left, module.insort_right):

    try:
        f(10, 10)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("TestErrorHandlingPython::test_non_sequence: ok")
"###);
    assert_output(&out, r###"TestErrorHandlingPython::test_non_sequence: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_insort_c__test_backcompatibility.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_insort_c__test_backcompatibility() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_insort_c__test_backcompatibility"
# subject = "cpython.test_bisect.TestInsortC.test_backcompatibility"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestInsortC::test_backcompatibility
"""Auto-ported test: TestInsortC::test_backcompatibility (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect

assert module.insort == module.insort_right
print("TestInsortC::test_backcompatibility: ok")
"###);
    assert_output(&out, r###"TestInsortC::test_backcompatibility: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_insort_c__test_list_derived.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_insort_c__test_list_derived() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_insort_c__test_list_derived"
# subject = "cpython.test_bisect.TestInsortC.test_listDerived"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestInsortC::test_listDerived
"""Auto-ported test: TestInsortC::test_listDerived (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect

class List(list):
    data = []

    def insert(self, index, item):
        self.data.insert(index, item)
lst = List()
module.insort_left(lst, 10)
module.insort_right(lst, 5)

assert [5, 10] == lst.data
print("TestInsortC::test_listDerived: ok")
"###);
    assert_output(&out, r###"TestInsortC::test_listDerived: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_insort_python__test_backcompatibility.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_insort_python__test_backcompatibility() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_insort_python__test_backcompatibility"
# subject = "cpython.test_bisect.TestInsortPython.test_backcompatibility"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestInsortPython::test_backcompatibility
"""Auto-ported test: TestInsortPython::test_backcompatibility (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect

assert module.insort == module.insort_right
print("TestInsortPython::test_backcompatibility: ok")
"###);
    assert_output(&out, r###"TestInsortPython::test_backcompatibility: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bisect/test_insort_python__test_list_derived.py`.
#[test]
fn test_gen_behavior_std_libs_bisect_test_insort_python__test_list_derived() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_insort_python__test_list_derived"
# subject = "cpython.test_bisect.TestInsortPython.test_listDerived"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestInsortPython::test_listDerived
"""Auto-ported test: TestInsortPython::test_listDerived (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = py_bisect

class List(list):
    data = []

    def insert(self, index, item):
        self.data.insert(index, item)
lst = List()
module.insort_left(lst, 10)
module.insort_right(lst, 5)

assert [5, 10] == lst.data
print("TestInsortPython::test_listDerived: ok")
"###);
    assert_output(&out, r###"TestInsortPython::test_listDerived: ok
"###);
}

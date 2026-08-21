# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "filterfalse_keeps_falsy"
# subject = "itertools.filterfalse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.filterfalse: filterfalse keeps items where the predicate is falsy (complement of filter)"""
import itertools

assert list(itertools.filterfalse(lambda x: x % 2, range(6))) == [0, 2, 4], "filterfalse evens"
assert list(itertools.filterfalse(lambda x: x % 2 == 0, [1, 2, 3, 4, 5])) == [1, 3, 5], "filterfalse odds"
assert list(itertools.filterfalse(lambda x: x, [0, 1, 0, 2, 0])) == [0, 0, 0], "filterfalse falsy"
assert list(itertools.filterfalse(lambda x: x, [])) == [], "filterfalse empty"

print("filterfalse_keeps_falsy OK")

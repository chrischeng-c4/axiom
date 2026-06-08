# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "compress_selects_by_truthy_mask"
# subject = "itertools.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.compress: compress yields data items whose selector is truthy, stopping at the shorter of data/selectors"""
import itertools

assert list(itertools.compress("ABCD", [1, 0, 1, 0])) == ["A", "C"], "compress basic"
assert list(itertools.compress("AB", [1, 1, 1, 1, 1])) == ["A", "B"], "compress stops at data"
assert list(itertools.compress([1, 2, 3], [True, False, True])) == [1, 3], "compress bool mask"
assert list(itertools.compress([1, 2, 3], ["x", "", "y"])) == [1, 3], "compress truthy strings"
assert list(itertools.compress([1, 2, 3], [0, 0, 0])) == [], "compress all false"

print("compress_selects_by_truthy_mask OK")

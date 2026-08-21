# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "chain_concatenates_iterables"
# subject = "itertools.chain"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.chain: chain flattens its positional iterables in order, including mixed list/tuple/str and empty inputs"""
import itertools

assert list(itertools.chain([1, 2], [3, 4], [5])) == [1, 2, 3, 4, 5], "chain three"
assert list(itertools.chain("abc", "def")) == ["a", "b", "c", "d", "e", "f"], "chain strings"
assert list(itertools.chain([1, 2], (3, 4), [5])) == [1, 2, 3, 4, 5], "chain mixed types"
assert list(itertools.chain()) == [], "chain no args"
assert list(itertools.chain([], [])) == [], "chain empty"
assert list(itertools.chain([1, 2, 3], [])) == [1, 2, 3], "chain trailing empty"
assert list(itertools.chain("")) == [], "chain empty string"

print("chain_concatenates_iterables OK")

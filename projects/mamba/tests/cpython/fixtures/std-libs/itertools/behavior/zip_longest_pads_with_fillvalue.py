# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "zip_longest_pads_with_fillvalue"
# subject = "itertools.zip_longest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.zip_longest: zip_longest pads every short column to the longest length, default fill None, custom via fillvalue"""
import itertools

z = list(itertools.zip_longest([1, 2, 3], [4, 5], fillvalue=0))
assert z == [(1, 4), (2, 5), (3, 0)], f"zip_longest = {z!r}"

default = list(itertools.zip_longest([1, 2], [3, 4, 5]))
assert default == [(1, 3), (2, 4), (None, 5)], f"default fill = {default!r}"

multi = list(itertools.zip_longest(range(3), range(1), range(2)))
assert multi == [(0, 0, 0), (1, None, 1), (2, None, None)], f"zip_longest multi = {multi!r}"

print("zip_longest_pads_with_fillvalue OK")

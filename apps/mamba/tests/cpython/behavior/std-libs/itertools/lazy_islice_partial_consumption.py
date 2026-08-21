# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "lazy_islice_partial_consumption"
# subject = "itertools.islice"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.islice: islice over a live iterator consumes exactly the requested window and leaves the remainder on the same source"""
import itertools

it = iter(range(10))
assert list(itertools.islice(it, 3)) == [0, 1, 2], "islice prefix"
assert list(it) == [3, 4, 5, 6, 7, 8, 9], "remainder after islice"

it = iter(range(10))
assert list(itertools.islice(it, 3, 3)) == [], "empty slice"
assert list(it) == [3, 4, 5, 6, 7, 8, 9], "untouched remainder"

c = itertools.count()
assert list(itertools.islice(c, 1, 3, 50)) == [1], "single hit"
assert next(c) == 3, "count position after islice"

print("lazy_islice_partial_consumption OK")

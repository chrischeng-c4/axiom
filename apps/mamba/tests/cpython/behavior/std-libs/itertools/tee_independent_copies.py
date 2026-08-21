# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "tee_independent_copies"
# subject = "itertools.tee"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.tee: tee returns independent iterators over the same source; exhausting one leaves the other intact"""
import itertools

a, b = itertools.tee([1, 2, 3])
assert list(a) == [1, 2, 3], f"tee a = {list(a)!r}"

# Exhausting one branch leaves the other intact.
a2, b2 = itertools.tee([1, 2, 3])
list(a2)  # exhaust a2
assert list(b2) == [1, 2, 3], f"tee b independent = {list(b2)!r}"

print("tee_independent_copies OK")

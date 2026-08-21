# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "repeat_finite_and_infinite"
# subject = "itertools.repeat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.repeat: repeat(obj, n) yields obj n times; repeat(obj) is infinite (one next() returns obj)"""
import itertools

assert list(itertools.repeat(5, 3)) == [5, 5, 5], "repeat finite"
assert list(itertools.repeat("x", 0)) == [], "repeat zero"
assert next(itertools.repeat(99)) == 99, "repeat infinite"

print("repeat_finite_and_infinite OK")

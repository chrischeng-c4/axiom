# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "reduce_fold_with_and_without_initial"
# subject = "functools.reduce"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.reduce: fold a binary op over a list with and without an initial seed (sum, product, max), and a single-element / empty+initial short-circuit that never calls func"""
import functools

# Fold without an initial seed.
assert functools.reduce(lambda a, b: a + b, [1, 2, 3, 4]) == 10, "reduce sum"
assert functools.reduce(lambda a, b: a * b, [1, 2, 3, 4]) == 24, "reduce product"
assert functools.reduce(max, [3, 1, 4, 1, 5, 9]) == 9, "reduce max"

# Fold with an initial seed.
assert functools.reduce(lambda a, b: a + b, [1, 2, 3], 100) == 106, "reduce with initial"
assert functools.reduce(lambda a, b: a + b, [], 42) == 42, "empty + initial returns seed"

# A single-element sequence returns that element without calling func.
assert functools.reduce(42, "1") == "1", "single element skips func"
# An empty sequence with an initial likewise never calls func.
assert functools.reduce(42, "", "1") == "1", "empty + initial skips func"

print("reduce_fold_with_and_without_initial OK")

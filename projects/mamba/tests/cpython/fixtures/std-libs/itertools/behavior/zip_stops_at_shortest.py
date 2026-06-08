# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "zip_stops_at_shortest"
# subject = "itertools.count"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.count: builtin zip pairs with count() and stops at the shortest input"""
import itertools

assert list(zip("abc", itertools.count())) == [("a", 0), ("b", 1), ("c", 2)], "zip+count"
assert list(zip("abcdef", range(3))) == [("a", 0), ("b", 1), ("c", 2)], "zip shortest"

print("zip_stops_at_shortest OK")

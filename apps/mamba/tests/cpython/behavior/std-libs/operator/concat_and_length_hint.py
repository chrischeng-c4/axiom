# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "concat_and_length_hint"
# subject = "operator.concat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.concat: concat joins two sequences of the same type (str/list/tuple) and length_hint reports the size of sized objects and ranges, with 0 for empty"""
import operator

assert operator.concat("hello ", "world") == "hello world", "concat str"
assert operator.concat([1, 2], [3, 4]) == [1, 2, 3, 4], "concat list"
assert operator.concat((1,), (2, 3)) == (1, 2, 3), "concat tuple"
assert operator.length_hint([]) == 0, "length_hint empty"
assert operator.length_hint([1, 2, 3]) == 3, "length_hint list"
assert operator.length_hint(range(10)) == 10, "length_hint range"

print("concat_and_length_hint OK")

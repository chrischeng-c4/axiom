# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "shallow_dict_shares_values"
# subject = "copy.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: shallow copy of a dict makes a new outer mapping but shares the mutable values"""
import copy

original = {"x": [1, 2], "y": [3, 4]}
shallow = copy.copy(original)
assert shallow == original and shallow is not original, "dict shallow outer is new"
assert shallow["x"] is original["x"], "dict shallow value is shared"

print("shallow_dict_shares_values OK")

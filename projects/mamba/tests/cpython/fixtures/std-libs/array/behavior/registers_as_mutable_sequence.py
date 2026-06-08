# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "registers_as_mutable_sequence"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: an array is an instance of collections.abc.MutableSequence and collections.abc.Reversible"""
import array

import collections.abc

a = array.array("i", [1, 2, 3])
assert isinstance(a, collections.abc.MutableSequence), "array is MutableSequence"
assert isinstance(a, collections.abc.Reversible), "array is Reversible"

print("registers_as_mutable_sequence OK")

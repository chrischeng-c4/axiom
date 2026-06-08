# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "iter_unpack_length_hint"
# subject = "struct.iter_unpack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.iter_unpack: operator.length_hint on a struct unpack-iterator reports the count of records still to yield, decrementing to 0 as the iterator drains"""
import operator
import struct

data = bytes(range(1, 16))  # three ">IB" records of 5 bytes each
s = struct.Struct(">IB")

# length_hint reports the number of records still to yield, draining to 0.
it = s.iter_unpack(data)
assert operator.length_hint(it) == 3, "length_hint start"
next(it)
assert operator.length_hint(it) == 2, "length_hint after one"
next(it)
next(it)
assert operator.length_hint(it) == 0, "length_hint exhausted"

print("iter_unpack_length_hint OK")

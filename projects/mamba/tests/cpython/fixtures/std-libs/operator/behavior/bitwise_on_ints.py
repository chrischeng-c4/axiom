# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "bitwise_on_ints"
# subject = "operator.and_"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.and_: the bitwise functions and_/or_/xor/lshift/rshift compute the same integer results as &, |, ^, <<, >>"""
import operator

assert operator.and_(0xFF, 0x0F) == 0x0F, "and_"
assert operator.or_(0xF0, 0x0F) == 0xFF, "or_"
assert operator.xor(0xFF, 0x0F) == 0xF0, "xor"
assert operator.lshift(1, 10) == 1024, "lshift"
assert operator.rshift(1024, 3) == 128, "rshift"

print("bitwise_on_ints OK")

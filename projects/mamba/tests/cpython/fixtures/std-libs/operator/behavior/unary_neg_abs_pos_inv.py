# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "unary_neg_abs_pos_inv"
# subject = "operator.neg"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.neg: the unary functions neg/abs/pos/inv compute -x, |x|, +x, ~x respectively over positive, negative and boundary integers"""
import operator

assert operator.neg(42) == -42, "neg positive"
assert operator.neg(-42) == 42, "neg negative"
assert operator.abs(-7) == 7, "abs negative"
assert operator.abs(7) == 7, "abs positive"
assert operator.pos(-3) == -3, "pos"
assert operator.inv(0) == -1, "inv 0"
assert operator.inv(-1) == 0, "inv -1"
assert operator.inv(5) == -6, "inv 5"

print("unary_neg_abs_pos_inv OK")

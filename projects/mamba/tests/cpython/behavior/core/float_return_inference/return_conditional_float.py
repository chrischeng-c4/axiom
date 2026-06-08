# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_conditional_float"
# subject = "conditional-branch float return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function whose both branches return a computed float must yield correct floats."""


def pick(x):
    if x > 0:
        return x / 2
    else:
        return x * 1.5


pos = pick(9)
neg = pick(-4)
assert pos == 4.5, pos
assert neg == -6.0, neg
assert isinstance(pos, float), type(pos)
assert isinstance(neg, float), type(neg)
print("return_conditional_float OK")

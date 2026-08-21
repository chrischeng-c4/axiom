# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "bound_arguments_equality_ignores_kwarg_order"
# subject = "inspect.Signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Signature: BoundArguments equality is by (signature, arguments) and ignores kwarg order; differing values are unequal"""
import inspect

def kw(*, a, b):
    pass

sigk = inspect.signature(kw)
b1 = sigk.bind(a=1, b=2)
b2 = sigk.bind(b=2, a=1)
assert b1 == b2, "kwarg-order-independent equality"

b3 = sigk.bind(a=1, b=3)
assert b1 != b3, "differing values not equal"

print("bound_arguments_equality_ignores_kwarg_order OK")

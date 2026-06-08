# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_argcount_attrs"
# subject = "types.CodeType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: co_argcount / co_posonlyargcount / co_kwonlyargcount reflect the signature shape of `def f(a, b, *, z=1, w=2)`: 2 / 0 / 2"""
import types


def sample(a, b, *, z=1, w=2):
    x = a + b
    return x


co = sample.__code__
assert co.co_argcount == 2, f"co_argcount = {co.co_argcount}"
assert co.co_posonlyargcount == 0, f"co_posonlyargcount = {co.co_posonlyargcount}"
assert co.co_kwonlyargcount == 2, f"co_kwonlyargcount = {co.co_kwonlyargcount}"

print("code_argcount_attrs OK")

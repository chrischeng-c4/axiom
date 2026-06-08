# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_name_and_varnames_attrs"
# subject = "types.CodeType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: co_name is the function name, a local ('x') appears in co_varnames, and co_nlocals is positive"""
import types


def sample(a, b, *, z=1, w=2):
    x = a + b
    return x


co = sample.__code__
assert co.co_name == "sample", f"co_name = {co.co_name!r}"
assert "x" in co.co_varnames, f"co_varnames = {co.co_varnames!r}"
assert co.co_nlocals > 0, f"co_nlocals = {co.co_nlocals}"

print("code_name_and_varnames_attrs OK")

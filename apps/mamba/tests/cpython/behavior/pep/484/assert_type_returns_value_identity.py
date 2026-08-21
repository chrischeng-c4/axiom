# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "assert_type_returns_value_identity"
# subject = "typing.assert_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.assert_type: assert_type returns the value object itself (identity), never a copy: assert_type(arg,object) is arg, assert_type(arg,str|float) is arg, assert_type(arg,None) is arg"""
from typing import assert_type

# assert_type returns the value object itself (identity), never a copy.
arg = object()
assert assert_type(arg, object) is arg
assert assert_type(arg, str | float) is arg
assert assert_type(arg, None) is arg

print("assert_type_returns_value_identity OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simplenamespace_nested_by_reference"
# subject = "types.SimpleNamespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: a nested SimpleNamespace is stored by reference, not copied, so the inner object is reachable via attribute access and identity-equal in vars()"""
import types

inner = types.SimpleNamespace(a=1, b=2)
outer = types.SimpleNamespace(x=inner)
assert outer.x.a == 1
assert vars(outer) == {"x": inner}
assert outer.x is inner

print("simplenamespace_nested_by_reference OK")

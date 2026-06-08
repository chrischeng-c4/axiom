# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "simplenamespace_is_callable"
# subject = "types.SimpleNamespace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.SimpleNamespace: simplenamespace_is_callable (surface)."""
import types

assert callable(types.SimpleNamespace)
print("simplenamespace_is_callable OK")

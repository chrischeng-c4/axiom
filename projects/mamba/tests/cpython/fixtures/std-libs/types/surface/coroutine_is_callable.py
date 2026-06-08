# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "coroutine_is_callable"
# subject = "types.coroutine"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.coroutine: coroutine_is_callable (surface)."""
import types

assert callable(types.coroutine)
print("coroutine_is_callable OK")

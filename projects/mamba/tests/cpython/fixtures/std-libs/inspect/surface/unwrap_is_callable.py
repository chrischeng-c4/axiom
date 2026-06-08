# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "unwrap_is_callable"
# subject = "inspect.unwrap"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.unwrap: unwrap_is_callable (surface)."""
import inspect

assert callable(inspect.unwrap)
print("unwrap_is_callable OK")

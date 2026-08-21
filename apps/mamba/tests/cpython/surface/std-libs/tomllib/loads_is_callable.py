# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "surface"
# case = "loads_is_callable"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tomllib.loads: loads_is_callable (surface)."""
import tomllib

assert callable(tomllib.loads)
print("loads_is_callable OK")

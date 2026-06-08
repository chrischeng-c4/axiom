# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "surface"
# case = "load_is_callable"
# subject = "tomllib.load"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tomllib.load: load_is_callable (surface)."""
import tomllib

assert callable(tomllib.load)
print("load_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_false_is_present"
# subject = "builtins.False"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.False: api_false_is_present (surface)."""
import builtins

assert hasattr(builtins, "False")
print("api_false_is_present OK")

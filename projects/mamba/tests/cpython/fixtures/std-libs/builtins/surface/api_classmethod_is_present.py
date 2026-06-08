# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_classmethod_is_present"
# subject = "builtins.classmethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.classmethod: api_classmethod_is_present (surface)."""
import builtins

assert hasattr(builtins, "classmethod")
print("api_classmethod_is_present OK")

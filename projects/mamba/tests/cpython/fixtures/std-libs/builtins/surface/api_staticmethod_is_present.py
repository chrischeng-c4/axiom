# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_staticmethod_is_present"
# subject = "builtins.staticmethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.staticmethod: api_staticmethod_is_present (surface)."""
import builtins

assert hasattr(builtins, "staticmethod")
print("api_staticmethod_is_present OK")

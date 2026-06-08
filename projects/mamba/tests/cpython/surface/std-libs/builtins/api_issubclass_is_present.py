# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_issubclass_is_present"
# subject = "builtins.issubclass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.issubclass: api_issubclass_is_present (surface)."""
import builtins

assert hasattr(builtins, "issubclass")
print("api_issubclass_is_present OK")

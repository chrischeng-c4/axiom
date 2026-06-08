# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_locals_is_present"
# subject = "builtins.locals"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.locals: api_locals_is_present (surface)."""
import builtins

assert hasattr(builtins, "locals")
print("api_locals_is_present OK")

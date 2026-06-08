# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_ellipsis_is_present"
# subject = "builtins.Ellipsis"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.Ellipsis: api_ellipsis_is_present (surface)."""
import builtins

assert hasattr(builtins, "Ellipsis")
print("api_ellipsis_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_exception_group_is_present"
# subject = "builtins.ExceptionGroup"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.ExceptionGroup: api_exception_group_is_present (surface)."""
import builtins

assert hasattr(builtins, "ExceptionGroup")
print("api_exception_group_is_present OK")

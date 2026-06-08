# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_base_exception_is_present"
# subject = "builtins.BaseException"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.BaseException: api_base_exception_is_present (surface)."""
import builtins

assert hasattr(builtins, "BaseException")
print("api_base_exception_is_present OK")

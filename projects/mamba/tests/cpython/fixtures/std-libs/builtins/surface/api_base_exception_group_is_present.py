# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_base_exception_group_is_present"
# subject = "builtins.BaseExceptionGroup"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.BaseExceptionGroup: api_base_exception_group_is_present (surface)."""
import builtins

assert hasattr(builtins, "BaseExceptionGroup")
print("api_base_exception_group_is_present OK")

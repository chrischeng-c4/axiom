# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_memoryview_is_present"
# subject = "builtins.memoryview"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.memoryview: api_memoryview_is_present (surface)."""
import builtins

assert hasattr(builtins, "memoryview")
print("api_memoryview_is_present OK")

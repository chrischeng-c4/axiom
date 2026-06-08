# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_isinstance_is_present"
# subject = "builtins.isinstance"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.isinstance: api_isinstance_is_present (surface)."""
import builtins

assert hasattr(builtins, "isinstance")
print("api_isinstance_is_present OK")

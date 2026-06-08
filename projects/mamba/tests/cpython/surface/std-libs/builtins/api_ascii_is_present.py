# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_ascii_is_present"
# subject = "builtins.ascii"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.ascii: api_ascii_is_present (surface)."""
import builtins

assert hasattr(builtins, "ascii")
print("api_ascii_is_present OK")

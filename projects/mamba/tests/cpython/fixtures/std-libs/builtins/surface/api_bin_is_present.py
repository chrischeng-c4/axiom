# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_bin_is_present"
# subject = "builtins.bin"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.bin: api_bin_is_present (surface)."""
import builtins

assert hasattr(builtins, "bin")
print("api_bin_is_present OK")

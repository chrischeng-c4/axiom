# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_chr_is_present"
# subject = "builtins.chr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.chr: api_chr_is_present (surface)."""
import builtins

assert hasattr(builtins, "chr")
print("api_chr_is_present OK")

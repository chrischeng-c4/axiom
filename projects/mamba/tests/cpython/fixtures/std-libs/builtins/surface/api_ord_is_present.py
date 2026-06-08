# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_ord_is_present"
# subject = "builtins.ord"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.ord: api_ord_is_present (surface)."""
import builtins

assert hasattr(builtins, "ord")
print("api_ord_is_present OK")

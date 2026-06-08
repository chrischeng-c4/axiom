# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_pow_is_present"
# subject = "builtins.pow"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.pow: api_pow_is_present (surface)."""
import builtins

assert hasattr(builtins, "pow")
print("api_pow_is_present OK")

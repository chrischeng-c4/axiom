# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_divmod_is_present"
# subject = "builtins.divmod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.divmod: api_divmod_is_present (surface)."""
import builtins

assert hasattr(builtins, "divmod")
print("api_divmod_is_present OK")

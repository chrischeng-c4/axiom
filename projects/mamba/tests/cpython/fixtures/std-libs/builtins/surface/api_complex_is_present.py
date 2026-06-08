# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_complex_is_present"
# subject = "builtins.complex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.complex: api_complex_is_present (surface)."""
import builtins

assert hasattr(builtins, "complex")
print("api_complex_is_present OK")

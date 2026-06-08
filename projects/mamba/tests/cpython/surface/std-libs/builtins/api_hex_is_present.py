# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_hex_is_present"
# subject = "builtins.hex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.hex: api_hex_is_present (surface)."""
import builtins

assert hasattr(builtins, "hex")
print("api_hex_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_bytes_is_present"
# subject = "builtins.bytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.bytes: api_bytes_is_present (surface)."""
import builtins

assert hasattr(builtins, "bytes")
print("api_bytes_is_present OK")

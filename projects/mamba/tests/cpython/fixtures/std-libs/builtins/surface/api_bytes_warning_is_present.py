# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_bytes_warning_is_present"
# subject = "builtins.BytesWarning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.BytesWarning: api_bytes_warning_is_present (surface)."""
import builtins

assert hasattr(builtins, "BytesWarning")
print("api_bytes_warning_is_present OK")

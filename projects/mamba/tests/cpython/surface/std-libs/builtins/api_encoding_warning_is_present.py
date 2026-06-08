# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_encoding_warning_is_present"
# subject = "builtins.EncodingWarning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.EncodingWarning: api_encoding_warning_is_present (surface)."""
import builtins

assert hasattr(builtins, "EncodingWarning")
print("api_encoding_warning_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_unicode_decode_error_is_present"
# subject = "builtins.UnicodeDecodeError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.UnicodeDecodeError: api_unicode_decode_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "UnicodeDecodeError")
print("api_unicode_decode_error_is_present OK")

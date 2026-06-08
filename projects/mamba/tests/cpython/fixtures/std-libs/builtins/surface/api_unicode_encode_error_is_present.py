# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_unicode_encode_error_is_present"
# subject = "builtins.UnicodeEncodeError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.UnicodeEncodeError: api_unicode_encode_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "UnicodeEncodeError")
print("api_unicode_encode_error_is_present OK")

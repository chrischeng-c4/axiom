# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_unicode_error_is_present"
# subject = "builtins.UnicodeError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.UnicodeError: api_unicode_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "UnicodeError")
print("api_unicode_error_is_present OK")

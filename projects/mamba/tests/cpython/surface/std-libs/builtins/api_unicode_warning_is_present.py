# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_unicode_warning_is_present"
# subject = "builtins.UnicodeWarning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.UnicodeWarning: api_unicode_warning_is_present (surface)."""
import builtins

assert hasattr(builtins, "UnicodeWarning")
print("api_unicode_warning_is_present OK")

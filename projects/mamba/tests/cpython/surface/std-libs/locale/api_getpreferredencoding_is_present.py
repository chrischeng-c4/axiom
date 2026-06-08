# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_getpreferredencoding_is_present"
# subject = "locale.getpreferredencoding"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.getpreferredencoding: api_getpreferredencoding_is_present (surface)."""
import locale

assert hasattr(locale, "getpreferredencoding")
print("api_getpreferredencoding_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_getencoding_is_present"
# subject = "locale.getencoding"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.getencoding: api_getencoding_is_present (surface)."""
import locale

assert hasattr(locale, "getencoding")
print("api_getencoding_is_present OK")

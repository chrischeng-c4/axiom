# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_locale_is_present"
# subject = "re.LOCALE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.LOCALE: api_locale_is_present (surface)."""
import re

assert hasattr(re, "LOCALE")
print("api_locale_is_present OK")

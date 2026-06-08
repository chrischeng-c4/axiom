# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_strxfrm_is_present"
# subject = "locale.strxfrm"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.strxfrm: api_strxfrm_is_present (surface)."""
import locale

assert hasattr(locale, "strxfrm")
print("api_strxfrm_is_present OK")

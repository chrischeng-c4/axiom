# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_strcoll_is_present"
# subject = "locale.strcoll"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.strcoll: api_strcoll_is_present (surface)."""
import locale

assert hasattr(locale, "strcoll")
print("api_strcoll_is_present OK")

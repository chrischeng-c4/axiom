# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "surface"
# case = "api_find_is_present"
# subject = "gettext.find"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gettext.find: api_find_is_present (surface)."""
import gettext

assert hasattr(gettext, "find")
print("api_find_is_present OK")

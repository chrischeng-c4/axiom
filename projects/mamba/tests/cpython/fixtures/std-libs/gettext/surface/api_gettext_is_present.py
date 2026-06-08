# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "surface"
# case = "api_gettext_is_present"
# subject = "gettext.gettext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gettext.gettext: api_gettext_is_present (surface)."""
import gettext

assert hasattr(gettext, "gettext")
print("api_gettext_is_present OK")

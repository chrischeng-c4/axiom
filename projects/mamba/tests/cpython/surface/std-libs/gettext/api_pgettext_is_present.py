# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "surface"
# case = "api_pgettext_is_present"
# subject = "gettext.pgettext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gettext.pgettext: api_pgettext_is_present (surface)."""
import gettext

assert hasattr(gettext, "pgettext")
print("api_pgettext_is_present OK")

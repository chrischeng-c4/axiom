# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "surface"
# case = "api_ngettext_is_present"
# subject = "gettext.ngettext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gettext.ngettext: api_ngettext_is_present (surface)."""
import gettext

assert hasattr(gettext, "ngettext")
print("api_ngettext_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "surface"
# case = "api_npgettext_is_present"
# subject = "gettext.npgettext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gettext.npgettext: api_npgettext_is_present (surface)."""
import gettext

assert hasattr(gettext, "npgettext")
print("api_npgettext_is_present OK")

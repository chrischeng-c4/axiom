# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "surface"
# case = "api_dpgettext_is_present"
# subject = "gettext.dpgettext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gettext.dpgettext: api_dpgettext_is_present (surface)."""
import gettext

assert hasattr(gettext, "dpgettext")
print("api_dpgettext_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "surface"
# case = "api_dngettext_is_present"
# subject = "gettext.dngettext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gettext.dngettext: api_dngettext_is_present (surface)."""
import gettext

assert hasattr(gettext, "dngettext")
print("api_dngettext_is_present OK")

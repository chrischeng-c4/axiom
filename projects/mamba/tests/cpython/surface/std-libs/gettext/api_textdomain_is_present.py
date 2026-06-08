# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "surface"
# case = "api_textdomain_is_present"
# subject = "gettext.textdomain"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gettext.textdomain: api_textdomain_is_present (surface)."""
import gettext

assert hasattr(gettext, "textdomain")
print("api_textdomain_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "surface"
# case = "api_bindtextdomain_is_present"
# subject = "gettext.bindtextdomain"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gettext.bindtextdomain: api_bindtextdomain_is_present (surface)."""
import gettext

assert hasattr(gettext, "bindtextdomain")
print("api_bindtextdomain_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "surface"
# case = "api_install_is_present"
# subject = "gettext.install"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gettext.install: api_install_is_present (surface)."""
import gettext

assert hasattr(gettext, "install")
print("api_install_is_present OK")

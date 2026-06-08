# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "surface"
# case = "api_catalog_is_present"
# subject = "gettext.Catalog"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gettext.Catalog: api_catalog_is_present (surface)."""
import gettext

assert hasattr(gettext, "Catalog")
print("api_catalog_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "surface"
# case = "api_kwlist_is_present"
# subject = "keyword.kwlist"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""keyword.kwlist: api_kwlist_is_present (surface)."""
import keyword

assert hasattr(keyword, "kwlist")
print("api_kwlist_is_present OK")

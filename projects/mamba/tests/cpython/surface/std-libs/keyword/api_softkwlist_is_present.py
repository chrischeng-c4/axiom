# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "surface"
# case = "api_softkwlist_is_present"
# subject = "keyword.softkwlist"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""keyword.softkwlist: api_softkwlist_is_present (surface)."""
import keyword

assert hasattr(keyword, "softkwlist")
print("api_softkwlist_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "surface"
# case = "api_sort_key_is_present"
# subject = "pstats.SortKey"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pstats.SortKey: api_sort_key_is_present (surface)."""
import pstats

assert hasattr(pstats, "SortKey")
print("api_sort_key_is_present OK")

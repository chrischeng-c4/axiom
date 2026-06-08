# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "api_s_iswht_is_present"
# subject = "stat.S_ISWHT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stat.S_ISWHT: api_s_iswht_is_present (surface)."""
import stat

assert hasattr(stat, "S_ISWHT")
print("api_s_iswht_is_present OK")

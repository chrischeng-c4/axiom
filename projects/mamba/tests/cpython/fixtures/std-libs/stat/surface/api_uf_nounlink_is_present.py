# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "api_uf_nounlink_is_present"
# subject = "stat.UF_NOUNLINK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stat.UF_NOUNLINK: api_uf_nounlink_is_present (surface)."""
import stat

assert hasattr(stat, "UF_NOUNLINK")
print("api_uf_nounlink_is_present OK")

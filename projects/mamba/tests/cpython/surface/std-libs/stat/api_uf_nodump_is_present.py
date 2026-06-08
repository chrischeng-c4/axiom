# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "api_uf_nodump_is_present"
# subject = "stat.UF_NODUMP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stat.UF_NODUMP: api_uf_nodump_is_present (surface)."""
import stat

assert hasattr(stat, "UF_NODUMP")
print("api_uf_nodump_is_present OK")

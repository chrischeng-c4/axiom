# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_hasconst_is_present"
# subject = "dis.hasconst"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.hasconst: api_hasconst_is_present (surface)."""
import dis

assert hasattr(dis, "hasconst")
print("api_hasconst_is_present OK")

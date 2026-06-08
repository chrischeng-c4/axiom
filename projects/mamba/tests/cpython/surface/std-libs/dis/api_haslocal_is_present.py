# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_haslocal_is_present"
# subject = "dis.haslocal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.haslocal: api_haslocal_is_present (surface)."""
import dis

assert hasattr(dis, "haslocal")
print("api_haslocal_is_present OK")

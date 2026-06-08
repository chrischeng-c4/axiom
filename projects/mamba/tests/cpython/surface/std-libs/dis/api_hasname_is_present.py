# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_hasname_is_present"
# subject = "dis.hasname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.hasname: api_hasname_is_present (surface)."""
import dis

assert hasattr(dis, "hasname")
print("api_hasname_is_present OK")

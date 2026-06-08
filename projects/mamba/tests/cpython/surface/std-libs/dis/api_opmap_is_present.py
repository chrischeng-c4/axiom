# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_opmap_is_present"
# subject = "dis.opmap"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.opmap: api_opmap_is_present (surface)."""
import dis

assert hasattr(dis, "opmap")
print("api_opmap_is_present OK")

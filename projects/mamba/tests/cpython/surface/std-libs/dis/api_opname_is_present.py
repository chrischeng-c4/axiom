# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_opname_is_present"
# subject = "dis.opname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.opname: api_opname_is_present (surface)."""
import dis

assert hasattr(dis, "opname")
print("api_opname_is_present OK")

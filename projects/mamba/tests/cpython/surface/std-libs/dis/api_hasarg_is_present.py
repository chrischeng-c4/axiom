# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_hasarg_is_present"
# subject = "dis.hasarg"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.hasarg: api_hasarg_is_present (surface)."""
import dis

assert hasattr(dis, "hasarg")
print("api_hasarg_is_present OK")

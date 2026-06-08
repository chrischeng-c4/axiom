# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_distb_is_present"
# subject = "dis.distb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.distb: api_distb_is_present (surface)."""
import dis

assert hasattr(dis, "distb")
print("api_distb_is_present OK")

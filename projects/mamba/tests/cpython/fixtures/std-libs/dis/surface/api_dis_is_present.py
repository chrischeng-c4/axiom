# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_dis_is_present"
# subject = "dis.dis"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.dis: api_dis_is_present (surface)."""
import dis

assert hasattr(dis, "dis")
print("api_dis_is_present OK")

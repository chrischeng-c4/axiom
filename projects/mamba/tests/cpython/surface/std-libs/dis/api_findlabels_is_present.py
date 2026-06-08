# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_findlabels_is_present"
# subject = "dis.findlabels"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.findlabels: api_findlabels_is_present (surface)."""
import dis

assert hasattr(dis, "findlabels")
print("api_findlabels_is_present OK")

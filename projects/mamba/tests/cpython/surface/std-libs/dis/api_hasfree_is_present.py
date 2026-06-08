# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_hasfree_is_present"
# subject = "dis.hasfree"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.hasfree: api_hasfree_is_present (surface)."""
import dis

assert hasattr(dis, "hasfree")
print("api_hasfree_is_present OK")

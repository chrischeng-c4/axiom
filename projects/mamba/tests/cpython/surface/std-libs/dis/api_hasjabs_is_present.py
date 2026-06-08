# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_hasjabs_is_present"
# subject = "dis.hasjabs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.hasjabs: api_hasjabs_is_present (surface)."""
import dis

assert hasattr(dis, "hasjabs")
print("api_hasjabs_is_present OK")

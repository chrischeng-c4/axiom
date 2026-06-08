# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_disco_is_present"
# subject = "dis.disco"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.disco: api_disco_is_present (surface)."""
import dis

assert hasattr(dis, "disco")
print("api_disco_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_hasexc_is_present"
# subject = "dis.hasexc"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.hasexc: api_hasexc_is_present (surface)."""
import dis

assert hasattr(dis, "hasexc")
print("api_hasexc_is_present OK")

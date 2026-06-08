# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "surface"
# case = "api_load_is_present"
# subject = "marshal.load"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""marshal.load: api_load_is_present (surface)."""
import marshal

assert hasattr(marshal, "load")
print("api_load_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "surface"
# case = "api_loads_is_present"
# subject = "marshal.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""marshal.loads: api_loads_is_present (surface)."""
import marshal

assert hasattr(marshal, "loads")
print("api_loads_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "surface"
# case = "api_close_is_present"
# subject = "fileinput.close"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""fileinput.close: api_close_is_present (surface)."""
import fileinput

assert hasattr(fileinput, "close")
print("api_close_is_present OK")

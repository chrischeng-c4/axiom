# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shelve"
# dimension = "surface"
# case = "api_open_is_present"
# subject = "shelve.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shelve.open: api_open_is_present (surface)."""
import shelve

assert hasattr(shelve, "open")
print("api_open_is_present OK")

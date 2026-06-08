# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "surface"
# case = "api_open_is_present"
# subject = "gzip.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gzip.open: api_open_is_present (surface)."""
import gzip

assert hasattr(gzip, "open")
print("api_open_is_present OK")

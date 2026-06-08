# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_open_is_present"
# subject = "tarfile.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.open: api_open_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "open")
print("api_open_is_present OK")

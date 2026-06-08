# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_read_error_is_present"
# subject = "tarfile.ReadError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.ReadError: api_read_error_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "ReadError")
print("api_read_error_is_present OK")

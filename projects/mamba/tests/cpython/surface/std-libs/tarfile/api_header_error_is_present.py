# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_header_error_is_present"
# subject = "tarfile.HeaderError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.HeaderError: api_header_error_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "HeaderError")
print("api_header_error_is_present OK")

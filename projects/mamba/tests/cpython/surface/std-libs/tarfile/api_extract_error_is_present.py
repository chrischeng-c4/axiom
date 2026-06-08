# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_extract_error_is_present"
# subject = "tarfile.ExtractError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.ExtractError: api_extract_error_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "ExtractError")
print("api_extract_error_is_present OK")

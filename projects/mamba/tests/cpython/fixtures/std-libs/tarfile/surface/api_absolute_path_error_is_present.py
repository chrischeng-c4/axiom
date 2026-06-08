# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_absolute_path_error_is_present"
# subject = "tarfile.AbsolutePathError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.AbsolutePathError: api_absolute_path_error_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "AbsolutePathError")
print("api_absolute_path_error_is_present OK")

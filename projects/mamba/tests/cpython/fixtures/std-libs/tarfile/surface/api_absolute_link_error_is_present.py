# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_absolute_link_error_is_present"
# subject = "tarfile.AbsoluteLinkError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.AbsoluteLinkError: api_absolute_link_error_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "AbsoluteLinkError")
print("api_absolute_link_error_is_present OK")

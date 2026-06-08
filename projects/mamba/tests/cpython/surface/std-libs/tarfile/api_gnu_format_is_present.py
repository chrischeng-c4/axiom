# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_gnu_format_is_present"
# subject = "tarfile.GNU_FORMAT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.GNU_FORMAT: api_gnu_format_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "GNU_FORMAT")
print("api_gnu_format_is_present OK")

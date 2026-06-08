# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_default_format_is_present"
# subject = "tarfile.DEFAULT_FORMAT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.DEFAULT_FORMAT: api_default_format_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "DEFAULT_FORMAT")
print("api_default_format_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_ustar_format_is_present"
# subject = "tarfile.USTAR_FORMAT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.USTAR_FORMAT: api_ustar_format_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "USTAR_FORMAT")
print("api_ustar_format_is_present OK")

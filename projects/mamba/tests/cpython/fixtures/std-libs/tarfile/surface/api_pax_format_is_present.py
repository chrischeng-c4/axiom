# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_pax_format_is_present"
# subject = "tarfile.PAX_FORMAT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.PAX_FORMAT: api_pax_format_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "PAX_FORMAT")
print("api_pax_format_is_present OK")

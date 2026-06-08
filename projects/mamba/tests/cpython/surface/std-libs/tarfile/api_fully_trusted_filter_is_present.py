# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_fully_trusted_filter_is_present"
# subject = "tarfile.fully_trusted_filter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.fully_trusted_filter: api_fully_trusted_filter_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "fully_trusted_filter")
print("api_fully_trusted_filter_is_present OK")

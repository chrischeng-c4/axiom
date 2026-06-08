# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_tar_filter_is_present"
# subject = "tarfile.tar_filter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.tar_filter: api_tar_filter_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "tar_filter")
print("api_tar_filter_is_present OK")

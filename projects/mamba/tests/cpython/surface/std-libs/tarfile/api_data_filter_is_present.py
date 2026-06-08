# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_data_filter_is_present"
# subject = "tarfile.data_filter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.data_filter: api_data_filter_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "data_filter")
print("api_data_filter_is_present OK")

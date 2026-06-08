# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_is_tarfile_is_present"
# subject = "tarfile.is_tarfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.is_tarfile: api_is_tarfile_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "is_tarfile")
print("api_is_tarfile_is_present OK")

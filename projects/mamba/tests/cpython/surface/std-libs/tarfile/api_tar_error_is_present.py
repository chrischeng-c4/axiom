# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_tar_error_is_present"
# subject = "tarfile.TarError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.TarError: api_tar_error_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "TarError")
print("api_tar_error_is_present OK")

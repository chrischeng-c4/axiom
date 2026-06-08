# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_tar_file_is_present"
# subject = "tarfile.TarFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.TarFile: api_tar_file_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "TarFile")
print("api_tar_file_is_present OK")

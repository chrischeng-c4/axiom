# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_tar_info_is_present"
# subject = "tarfile.TarInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.TarInfo: api_tar_info_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "TarInfo")
print("api_tar_info_is_present OK")

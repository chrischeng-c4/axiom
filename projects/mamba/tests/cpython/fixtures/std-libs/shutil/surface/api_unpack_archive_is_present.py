# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_unpack_archive_is_present"
# subject = "shutil.unpack_archive"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.unpack_archive: api_unpack_archive_is_present (surface)."""
import shutil

assert hasattr(shutil, "unpack_archive")
print("api_unpack_archive_is_present OK")

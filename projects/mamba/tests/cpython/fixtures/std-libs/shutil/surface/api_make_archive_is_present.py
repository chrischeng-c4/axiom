# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_make_archive_is_present"
# subject = "shutil.make_archive"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.make_archive: api_make_archive_is_present (surface)."""
import shutil

assert hasattr(shutil, "make_archive")
print("api_make_archive_is_present OK")

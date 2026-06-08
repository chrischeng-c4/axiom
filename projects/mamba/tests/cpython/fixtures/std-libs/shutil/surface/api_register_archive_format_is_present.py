# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_register_archive_format_is_present"
# subject = "shutil.register_archive_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.register_archive_format: api_register_archive_format_is_present (surface)."""
import shutil

assert hasattr(shutil, "register_archive_format")
print("api_register_archive_format_is_present OK")

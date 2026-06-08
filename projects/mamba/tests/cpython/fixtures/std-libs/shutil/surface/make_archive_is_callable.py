# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "make_archive_is_callable"
# subject = "shutil.make_archive"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shutil.make_archive: make_archive_is_callable (surface)."""
import shutil

assert callable(shutil.make_archive)
print("make_archive_is_callable OK")

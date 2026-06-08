# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "unpack_archive_is_callable"
# subject = "shutil.unpack_archive"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shutil.unpack_archive: unpack_archive_is_callable (surface)."""
import shutil

assert callable(shutil.unpack_archive)
print("unpack_archive_is_callable OK")

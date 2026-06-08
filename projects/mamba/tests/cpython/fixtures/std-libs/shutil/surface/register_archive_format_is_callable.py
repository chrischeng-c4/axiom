# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "register_archive_format_is_callable"
# subject = "shutil.register_archive_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shutil.register_archive_format: register_archive_format_is_callable (surface)."""
import shutil

assert callable(shutil.register_archive_format)
print("register_archive_format_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "register_archive_format_noncallable_raises"
# subject = "shutil.register_archive_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.register_archive_format: register_archive_format_noncallable_raises (errors)."""
import shutil

_raised = False
try:
    shutil.register_archive_format("mamba_bad_fmt", 1)
except TypeError:
    _raised = True
assert _raised, "register_archive_format_noncallable_raises: expected TypeError"
print("register_archive_format_noncallable_raises OK")

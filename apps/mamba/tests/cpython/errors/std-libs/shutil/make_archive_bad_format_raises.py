# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "make_archive_bad_format_raises"
# subject = "shutil.make_archive"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.make_archive: make_archive_bad_format_raises (errors)."""
import shutil

_raised = False
try:
    shutil.make_archive("/tmp/mamba_shutil_test_archive", "no_such_format")
except ValueError:
    _raised = True
assert _raised, "make_archive_bad_format_raises: expected ValueError"
print("make_archive_bad_format_raises OK")

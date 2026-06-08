# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "missing_path_raises_oserror"
# subject = "zipfile.ZipFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: missing_path_raises_oserror (errors)."""
import zipfile
import os
import tempfile

_raised = False
try:
    zipfile.ZipFile(os.path.join(tempfile.mkdtemp(), 'nope.zip'))
except OSError:
    _raised = True
assert _raised, "missing_path_raises_oserror: expected OSError"
print("missing_path_raises_oserror OK")

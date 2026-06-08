# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "errors"
# case = "named_file_bad_mode_raises"
# subject = "tempfile.NamedTemporaryFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: named_file_bad_mode_raises (errors)."""
import tempfile

_raised = False
try:
    tempfile.NamedTemporaryFile(mode='Q')
except ValueError:
    _raised = True
assert _raised, "named_file_bad_mode_raises: expected ValueError"
print("named_file_bad_mode_raises OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "errors"
# case = "mkstemp_nonexistent_dir_raises"
# subject = "tempfile.mkstemp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.mkstemp: mkstemp_nonexistent_dir_raises (errors)."""
import tempfile

_raised = False
try:
    tempfile.mkstemp(dir='/nonexistent_dir_xyzzy')
except FileNotFoundError:
    _raised = True
assert _raised, "mkstemp_nonexistent_dir_raises: expected FileNotFoundError"
print("mkstemp_nonexistent_dir_raises OK")

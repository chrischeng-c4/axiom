# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "open_missing_file_raises_filenotfound"
# subject = "tarfile.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile.open: open_missing_file_raises_filenotfound (errors)."""
import tarfile

_raised = False
try:
    tarfile.open('/no/such/file.tar')
except FileNotFoundError:
    _raised = True
assert _raised, "open_missing_file_raises_filenotfound: expected FileNotFoundError"
print("open_missing_file_raises_filenotfound OK")

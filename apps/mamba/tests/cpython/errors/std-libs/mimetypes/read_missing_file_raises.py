# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "errors"
# case = "read_missing_file_raises"
# subject = "mimetypes.MimeTypes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.MimeTypes: read_missing_file_raises (errors)."""
import mimetypes

_raised = False
try:
    mimetypes.MimeTypes().read('/no/such/mime.types')
except FileNotFoundError:
    _raised = True
assert _raised, "read_missing_file_raises: expected FileNotFoundError"
print("read_missing_file_raises OK")

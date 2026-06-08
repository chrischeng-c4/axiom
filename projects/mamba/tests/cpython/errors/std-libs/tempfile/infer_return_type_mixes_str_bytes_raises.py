# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "errors"
# case = "infer_return_type_mixes_str_bytes_raises"
# subject = "tempfile._infer_return_type"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile._infer_return_type: infer_return_type_mixes_str_bytes_raises (errors)."""
import tempfile

_raised = False
try:
    tempfile._infer_return_type('', b'')
except TypeError:
    _raised = True
assert _raised, "infer_return_type_mixes_str_bytes_raises: expected TypeError"
print("infer_return_type_mixes_str_bytes_raises OK")

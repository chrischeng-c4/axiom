# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "errors"
# case = "extract_tb_non_traceback_raises"
# subject = "traceback.extract_tb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.extract_tb: extract_tb_non_traceback_raises (errors)."""
import traceback

_raised = False
try:
    traceback.extract_tb(123)
except AttributeError:
    _raised = True
assert _raised, "extract_tb_non_traceback_raises: expected AttributeError"
print("extract_tb_non_traceback_raises OK")

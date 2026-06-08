# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "open_text_param_in_binary_mode_raises"
# subject = "bz2.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: open_text_param_in_binary_mode_raises (errors)."""
import bz2, io

_raised = False
try:
    bz2.open(io.BytesIO(), "rb", encoding="utf-8")
except ValueError:
    _raised = True
assert _raised, "open_text_param_in_binary_mode_raises: expected ValueError"
print("open_text_param_in_binary_mode_raises OK")

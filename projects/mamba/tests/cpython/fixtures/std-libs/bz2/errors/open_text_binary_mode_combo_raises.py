# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "open_text_binary_mode_combo_raises"
# subject = "bz2.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: open_text_binary_mode_combo_raises (errors)."""
import bz2, io

_raised = False
try:
    bz2.open(io.BytesIO(), "wbt")
except ValueError:
    _raised = True
assert _raised, "open_text_binary_mode_combo_raises: expected ValueError"
print("open_text_binary_mode_combo_raises OK")

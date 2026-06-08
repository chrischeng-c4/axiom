# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "errors"
# case = "convert_old_underscore_helper_gone"
# subject = "enum.IntEnum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.IntEnum: convert_old_underscore_helper_gone (errors)."""
import enum

_raised = False
try:
    enum.IntEnum._convert('X', __name__, filter=lambda n: False)
except AttributeError:
    _raised = True
assert _raised, "convert_old_underscore_helper_gone: expected AttributeError"
print("convert_old_underscore_helper_gone OK")

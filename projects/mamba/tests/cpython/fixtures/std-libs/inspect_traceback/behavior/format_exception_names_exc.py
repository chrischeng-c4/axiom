# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "behavior"
# case = "format_exception_names_exc"
# subject = "traceback.format_exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
"""traceback.format_exception: inside an except block, traceback.format_exception(*sys.exc_info()) returns a list of str lines and the active exception type name appears among them"""
import sys
import traceback

try:
    raise KeyError("missing")
except KeyError:
    lines = traceback.format_exception(*sys.exc_info())
    assert isinstance(lines, list)
    assert len(lines) > 0
    assert any("KeyError" in line for line in lines), lines

print("format_exception_names_exc OK")

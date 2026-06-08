# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_exc_captures_active_exception"
# subject = "traceback.format_exc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exc: inside an except block, format_exc() includes the 'Traceback (most recent call last):' header and the 'RuntimeError: runtime msg' type+message"""
import traceback

try:
    raise RuntimeError("runtime msg")
except RuntimeError:
    _fe = traceback.format_exc()
assert "RuntimeError: runtime msg" in _fe, f"format_exc has type+msg: {_fe!r}"
assert "Traceback (most recent call last):" in _fe, "has Traceback header"

print("format_exc_captures_active_exception OK")

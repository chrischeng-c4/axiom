# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "exc_info_inside_handler"
# subject = "sys.exc_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.exc_info: inside an except ValueError block, exc_info()[0] is ValueError and str(exc_info()[1]) is the raised message"""
import sys

try:
    raise ValueError("test_error")
except ValueError:
    _et, _ev, _etb = sys.exc_info()
    assert _et is ValueError, f"exc_type in handler = {_et!r}"
    assert str(_ev) == "test_error", f"exc_val = {str(_ev)!r}"
print("exc_info_inside_handler OK")

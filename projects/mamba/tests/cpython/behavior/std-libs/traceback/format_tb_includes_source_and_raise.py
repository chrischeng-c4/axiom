# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_tb_includes_source_and_raise"
# subject = "traceback.format_tb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_tb: format_tb of a live traceback joins to a string containing a '.py' filename and the offending 'raise TypeError' source statement"""
import sys
import traceback

try:
    raise TypeError("tb check")
except TypeError:
    _tb = sys.exc_info()[2]
    _tb_str = "".join(traceback.format_tb(_tb))
assert ".py" in _tb_str, f"format_tb has filename: {_tb_str!r}"
assert "raise TypeError" in _tb_str, f"format_tb has raise statement: {_tb_str!r}"

print("format_tb_includes_source_and_raise OK")

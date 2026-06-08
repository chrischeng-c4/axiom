# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "extract_tb_entries_have_frame_fields"
# subject = "traceback.extract_tb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.extract_tb: extract_tb of a live traceback yields StackSummary entries whose filename/lineno/name have str/int/str types"""
import sys
import traceback

try:
    raise StopIteration("stop")
except StopIteration:
    _ss = traceback.extract_tb(sys.exc_info()[2])
assert len(_ss) >= 1, "stack has frames"
_frame = _ss[-1]
assert isinstance(_frame.filename, str), "filename type"
assert isinstance(_frame.lineno, int), "lineno type"
assert isinstance(_frame.name, str), "name type"

print("extract_tb_entries_have_frame_fields OK")

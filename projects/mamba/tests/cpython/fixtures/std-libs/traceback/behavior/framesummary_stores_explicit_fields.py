# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "framesummary_stores_explicit_fields"
# subject = "traceback.FrameSummary"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.FrameSummary: FrameSummary('f', 1, 'dummy', line='line') stores filename/lineno/name/line verbatim and reports len() == 4 logical fields"""
import traceback

f = traceback.FrameSummary("f", 1, "dummy", line="line")
assert f.line == "line", f"explicit line = {f.line!r}"
assert f.filename == "f", f"filename = {f.filename!r}"
assert f.lineno == 1, f"lineno = {f.lineno!r}"
assert f.name == "dummy", f"name = {f.name!r}"
assert len(f) == 4, f"len(FrameSummary) = {len(f)!r}"

print("framesummary_stores_explicit_fields OK")

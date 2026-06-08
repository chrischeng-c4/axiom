# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "framesummary_missing_line_is_none"
# subject = "traceback.FrameSummary"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.FrameSummary: FrameSummary('f', None, 'dummy') with no lineno and no explicit line has .line is None (no source lookup)"""
import traceback

g = traceback.FrameSummary("f", None, "dummy")
assert g.line is None, f"missing line = {g.line!r}"

print("framesummary_missing_line_is_none OK")

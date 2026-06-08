# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "StackSummary__format_frame_summary__frame_summary_as_FrameSummary_wrong"
# subject = "traceback.StackSummary.format_frame_summary(frame_summary: FrameSummary)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.StackSummary.format_frame_summary(frame_summary: FrameSummary); call it with the wrong type.

typeshed contract: frame_summary is FrameSummary. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import StackSummary
obj = object.__new__(StackSummary)
try:
    obj.format_frame_summary(_W())  # frame_summary: FrameSummary <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

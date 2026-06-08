# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "stacksummary_subclass_format_frame_summary_hook"
# subject = "traceback.StackSummary"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.StackSummary: a StackSummary subclass that overrides format_frame_summary customizes each formatted line (filename:lineno) via extract(walk_stack(None), limit=1)"""
import traceback


class CompactSummary(traceback.StackSummary):
    def format_frame_summary(self, frame_summary):
        return f"{frame_summary.filename}:{frame_summary.lineno}"


def some_inner():
    return CompactSummary.extract(traceback.walk_stack(None), limit=1)


cs = some_inner()
_expected = f"{__file__}:{some_inner.__code__.co_firstlineno + 1}"
assert cs.format() == [_expected], f"custom format = {cs.format()!r}"

print("stacksummary_subclass_format_frame_summary_hook OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "stacksummary_extract_capture_locals"
# subject = "traceback.StackSummary"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.StackSummary: StackSummary.extract(..., capture_locals=True) stores frame locals as repr-strings ({'something': '1'}); without the flag .locals is None"""
import sys
import traceback


def make_frame():
    something = 1
    return sys._getframe()


fr = make_frame()
with_locals = traceback.StackSummary.extract(iter([(fr, fr.f_lineno)]), capture_locals=True)
assert with_locals[0].locals == {"something": "1"}, f"locals = {with_locals[0].locals!r}"
without = traceback.StackSummary.extract(iter([(fr, fr.f_lineno)]))
assert without[0].locals is None, f"no-capture locals = {without[0].locals!r}"

print("stacksummary_extract_capture_locals OK")

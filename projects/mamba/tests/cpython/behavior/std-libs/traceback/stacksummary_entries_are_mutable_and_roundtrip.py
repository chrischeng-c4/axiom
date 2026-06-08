# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "stacksummary_entries_are_mutable_and_roundtrip"
# subject = "traceback.StackSummary"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.StackSummary: StackSummary entries are mutable; editing s[0]'s lineno and feeding it back through from_list re-formats with the new line number"""
import traceback

s = traceback.StackSummary.from_list([("foo.py", 1, "fred", "line")])
s[0] = ("foo.py", 2, "fred", "line")
s2 = traceback.StackSummary.from_list(s)
assert s2.format() == ['  File "foo.py", line 2, in fred\n    line\n'], \
    f"edited format = {s2.format()!r}"

print("stacksummary_entries_are_mutable_and_roundtrip OK")

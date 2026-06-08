# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "stacksummary_from_list_formats_one_block_per_frame"
# subject = "traceback.StackSummary"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.StackSummary: StackSummary.from_list([('foo.py', 1, 'fred', 'line')]).format() yields one '  File "foo.py", line 1, in fred\\n    line\\n' block"""
import traceback

s = traceback.StackSummary.from_list([("foo.py", 1, "fred", "line")])
assert s.format() == ['  File "foo.py", line 1, in fred\n    line\n'], \
    f"from_list format = {s.format()!r}"

print("stacksummary_from_list_formats_one_block_per_frame OK")

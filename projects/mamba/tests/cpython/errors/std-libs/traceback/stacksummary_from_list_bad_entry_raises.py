# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "errors"
# case = "stacksummary_from_list_bad_entry_raises"
# subject = "traceback.StackSummary"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.StackSummary: stacksummary_from_list_bad_entry_raises (errors)."""
import traceback

_raised = False
try:
    traceback.StackSummary.from_list([42]).format()
except TypeError:
    _raised = True
assert _raised, "stacksummary_from_list_bad_entry_raises: expected TypeError"
print("stacksummary_from_list_bad_entry_raises OK")

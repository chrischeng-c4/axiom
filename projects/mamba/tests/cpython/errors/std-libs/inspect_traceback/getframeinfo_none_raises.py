# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "errors"
# case = "getframeinfo_none_raises"
# subject = "inspect.getframeinfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
"""inspect.getframeinfo: getframeinfo_none_raises (errors)."""
import inspect

_raised = False
try:
    inspect.getframeinfo(None)
except AttributeError:
    _raised = True
assert _raised, "getframeinfo_none_raises: expected AttributeError"
print("getframeinfo_none_raises OK")

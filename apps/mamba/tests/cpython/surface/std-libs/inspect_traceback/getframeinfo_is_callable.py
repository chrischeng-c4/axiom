# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "surface"
# case = "getframeinfo_is_callable"
# subject = "inspect.getframeinfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getframeinfo: getframeinfo_is_callable (surface)."""
import inspect

assert callable(inspect.getframeinfo)
print("getframeinfo_is_callable OK")

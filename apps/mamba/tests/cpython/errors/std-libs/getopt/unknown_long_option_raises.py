# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "errors"
# case = "unknown_long_option_raises"
# subject = "getopt.getopt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: unknown_long_option_raises (errors)."""
import getopt

_raised = False
try:
    getopt.getopt(['--unknown'], '', ['help'])
except getopt.GetoptError:
    _raised = True
assert _raised, "unknown_long_option_raises: expected getopt.GetoptError"
print("unknown_long_option_raises OK")

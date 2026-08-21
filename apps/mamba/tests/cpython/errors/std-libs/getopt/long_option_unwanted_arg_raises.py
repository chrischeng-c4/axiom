# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "errors"
# case = "long_option_unwanted_arg_raises"
# subject = "getopt.getopt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: long_option_unwanted_arg_raises (errors)."""
import getopt

_raised = False
try:
    getopt.getopt(['--flag=x'], '', ['flag'])
except getopt.GetoptError:
    _raised = True
assert _raised, "long_option_unwanted_arg_raises: expected getopt.GetoptError"
print("long_option_unwanted_arg_raises OK")

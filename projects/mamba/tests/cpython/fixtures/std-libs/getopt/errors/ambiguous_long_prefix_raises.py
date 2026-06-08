# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "errors"
# case = "ambiguous_long_prefix_raises"
# subject = "getopt.getopt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: ambiguous_long_prefix_raises (errors)."""
import getopt

_raised = False
try:
    getopt.getopt(['--he'], '', ['help', 'header'])
except getopt.GetoptError:
    _raised = True
assert _raised, "ambiguous_long_prefix_raises: expected getopt.GetoptError"
print("ambiguous_long_prefix_raises OK")

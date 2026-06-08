# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "errors"
# case = "missing_argument_raises"
# subject = "getopt.getopt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: missing_argument_raises (errors)."""
import getopt

_raised = False
try:
    getopt.getopt(['-b'], 'ab:')
except getopt.GetoptError:
    _raised = True
assert _raised, "missing_argument_raises: expected getopt.GetoptError"
print("missing_argument_raises OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "errors"
# case = "extra_positional_arg_raises"
# subject = "pprint.PrettyPrinter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.PrettyPrinter: extra_positional_arg_raises (errors)."""
import pprint

_raised = False
try:
    pprint.PrettyPrinter(4, 40, 5, __import__('io').StringIO(), True)
except TypeError:
    _raised = True
assert _raised, "extra_positional_arg_raises: expected TypeError"
print("extra_positional_arg_raises OK")

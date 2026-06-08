# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "errors"
# case = "negative_indent_raises"
# subject = "pprint.PrettyPrinter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pprint.PrettyPrinter: negative_indent_raises (errors)."""
import pprint

_raised = False
try:
    pprint.PrettyPrinter(indent=-1)
except ValueError:
    _raised = True
assert _raised, "negative_indent_raises: expected ValueError"
print("negative_indent_raises OK")

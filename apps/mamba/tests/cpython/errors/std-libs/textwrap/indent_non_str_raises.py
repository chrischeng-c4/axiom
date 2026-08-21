# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "errors"
# case = "indent_non_str_raises"
# subject = "textwrap.indent"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.indent: indent_non_str_raises (errors)."""
import textwrap

_raised = False
try:
    textwrap.indent(123, "  ")
except AttributeError:
    _raised = True
assert _raised, "indent_non_str_raises: expected AttributeError"
print("indent_non_str_raises OK")

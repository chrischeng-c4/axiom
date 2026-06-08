# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "errors"
# case = "dedent_non_str_raises"
# subject = "textwrap.dedent"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.dedent: dedent_non_str_raises (errors)."""
import textwrap

_raised = False
try:
    textwrap.dedent(123)
except TypeError:
    _raised = True
assert _raised, "dedent_non_str_raises: expected TypeError"
print("dedent_non_str_raises OK")

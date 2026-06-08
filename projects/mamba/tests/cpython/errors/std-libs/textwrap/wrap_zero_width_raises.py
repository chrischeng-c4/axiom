# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "errors"
# case = "wrap_zero_width_raises"
# subject = "textwrap.wrap"
# kind = "mechanical"
# xfail = "textwrap.wrap is a silent stub under mamba — width<=0 does not raise (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.wrap: wrap_zero_width_raises (errors)."""
import textwrap

_raised = False
try:
    textwrap.wrap("hello", width=0)
except ValueError:
    _raised = True
assert _raised, "wrap_zero_width_raises: expected ValueError"
print("wrap_zero_width_raises OK")

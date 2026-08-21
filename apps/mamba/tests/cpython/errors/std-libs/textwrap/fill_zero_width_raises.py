# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "errors"
# case = "fill_zero_width_raises"
# subject = "textwrap.fill"
# kind = "mechanical"
# xfail = "textwrap.fill is a silent stub under mamba — width<=0 does not raise (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.fill: fill_zero_width_raises (errors)."""
import textwrap

_raised = False
try:
    textwrap.fill("hello", width=0)
except ValueError:
    _raised = True
assert _raised, "fill_zero_width_raises: expected ValueError"
print("fill_zero_width_raises OK")

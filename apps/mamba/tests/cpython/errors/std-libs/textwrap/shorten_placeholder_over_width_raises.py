# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "errors"
# case = "shorten_placeholder_over_width_raises"
# subject = "textwrap.shorten"
# kind = "mechanical"
# xfail = "textwrap.shorten is a silent stub under mamba — placeholder-too-large does not raise (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.shorten: shorten_placeholder_over_width_raises (errors)."""
import textwrap

_raised = False
try:
    textwrap.shorten("x" * 20, width=8, placeholder="(.......)")
except ValueError:
    _raised = True
assert _raised, "shorten_placeholder_over_width_raises: expected ValueError"
print("shorten_placeholder_over_width_raises OK")

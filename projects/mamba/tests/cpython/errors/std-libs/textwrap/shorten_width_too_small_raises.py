# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "errors"
# case = "shorten_width_too_small_raises"
# subject = "textwrap.shorten"
# kind = "mechanical"
# xfail = "textwrap.shorten is a silent stub under mamba — placeholder-too-large does not raise (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.shorten: shorten_width_too_small_raises (errors)."""
import textwrap

_raised = False
try:
    textwrap.shorten("hello world", width=2, placeholder="[...]")
except ValueError:
    _raised = True
assert _raised, "shorten_width_too_small_raises: expected ValueError"
print("shorten_width_too_small_raises OK")

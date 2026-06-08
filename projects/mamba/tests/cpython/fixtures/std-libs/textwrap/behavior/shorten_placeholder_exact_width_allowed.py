# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "shorten_placeholder_exact_width_allowed"
# subject = "textwrap.shorten"
# kind = "semantic"
# xfail = "textwrap.shorten is a silent stub under mamba — does not return the placeholder (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.shorten: a placeholder exactly as wide as width is accepted and returned when the whole text is collapsed away"""
import textwrap

# An 8-char placeholder at width=8 is the boundary case: accepted and returned.
result = textwrap.shorten("x" * 20, width=8, placeholder="(......)")
assert result == "(......)", f"placeholder of exactly width chars is allowed: {result!r}"
print("shorten_placeholder_exact_width_allowed OK")

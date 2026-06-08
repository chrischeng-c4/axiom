# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "fill_joins_lines_within_width"
# subject = "textwrap.fill"
# kind = "semantic"
# xfail = "textwrap.fill is a silent stub under mamba — returns the input unchanged, no wrap (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.fill: fill returns a single newline-joined string whose every line is within width"""
import textwrap

filled = textwrap.fill("one two three four five", width=12)
assert isinstance(filled, str), f"fill type = {type(filled)!r}"
lines = filled.split("\n")
assert len(lines) > 1, f"expected multiple lines, got {lines!r}"
assert all(len(line) <= 12 for line in lines), f"fill lines <= 12: {lines!r}"
print("fill_joins_lines_within_width OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_predicate_selects_lines"
# subject = "textwrap.indent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.indent: indent with a predicate= callable prefixes only the lines for which the predicate returns True"""
import textwrap

out = textwrap.indent(
    "line1\nline2\nline3", "* ", predicate=lambda s: s.startswith("line2")
)
assert out == "line1\n* line2\nline3", f"indent predicate = {out!r}"
print("indent_predicate_selects_lines OK")

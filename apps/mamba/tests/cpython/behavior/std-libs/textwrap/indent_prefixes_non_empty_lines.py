# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_prefixes_non_empty_lines"
# subject = "textwrap.indent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.indent: indent prepends the prefix to every non-empty line by default, leaving empty lines untouched"""
import textwrap

# Default predicate prefixes non-empty lines only; the blank line is untouched.
out = textwrap.indent("line1\nline2\n\nline3", "# ")
assert out == "# line1\n# line2\n\n# line3", f"indent = {out!r}"
# Simple multi-line prefix with a trailing newline preserved.
out2 = textwrap.indent("line1\nline2\nline3", ">> ")
assert out2 == ">> line1\n>> line2\n>> line3", f"indent = {out2!r}"
print("indent_prefixes_non_empty_lines OK")

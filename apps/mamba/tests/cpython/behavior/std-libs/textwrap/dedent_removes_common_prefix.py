# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_removes_common_prefix"
# subject = "textwrap.dedent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.dedent: dedent strips the minimum common leading whitespace shared by all content lines"""
import textwrap

# A constant prefix is stripped from every content line.
even = "  Hello there.\n  How are ya?\n  Oh good."
assert textwrap.dedent(even) == "Hello there.\nHow are ya?\nOh good.", (
    f"even = {textwrap.dedent(even)!r}"
)
# The minimum (here 3 spaces) is removed; deeper lines keep the excess.
text = "   line1\n   line2\n     line3\n"
assert textwrap.dedent(text) == "line1\nline2\n  line3\n", (
    f"dedent result = {textwrap.dedent(text)!r}"
)
print("dedent_removes_common_prefix OK")

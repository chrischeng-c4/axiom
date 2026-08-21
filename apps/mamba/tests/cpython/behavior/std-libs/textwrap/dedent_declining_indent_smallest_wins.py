# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_declining_indent_smallest_wins"
# subject = "textwrap.dedent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.dedent: with declining indentation the smallest content prefix wins; blank/whitespace-only interior lines do not change the result"""
import textwrap

# The smallest content prefix (here one space) is the common prefix.
declining = "     Foo\n    Bar\n"
assert textwrap.dedent(declining) == " Foo\nBar\n", (
    f"declining = {textwrap.dedent(declining)!r}"
)
# Blank or whitespace-only interior lines do not change that result.
assert textwrap.dedent("     Foo\n\n    Bar\n") == " Foo\n\nBar\n", "blank interior"
assert textwrap.dedent("     Foo\n    \n    Bar\n") == " Foo\n\nBar\n", "ws interior"
print("dedent_declining_indent_smallest_wins OK")

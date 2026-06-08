# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_ignores_whitespace_only_lines"
# subject = "textwrap.dedent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.dedent: a whitespace-only line does not count toward the common prefix and is normalized to empty in the output"""
import textwrap

# The "  \n" line between content lines is whitespace-only: it is ignored when
# computing the common prefix and blanked to "\n" in the output.
ws_only = "  Hello there.\n  \n  How are ya?\n  Oh good.\n"
assert textwrap.dedent(ws_only) == "Hello there.\n\nHow are ya?\nOh good.\n", (
    f"ws_only = {textwrap.dedent(ws_only)!r}"
)
print("dedent_ignores_whitespace_only_lines OK")

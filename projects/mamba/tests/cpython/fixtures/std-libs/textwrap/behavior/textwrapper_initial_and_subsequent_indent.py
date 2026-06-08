# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "textwrapper_initial_and_subsequent_indent"
# subject = "textwrap.TextWrapper"
# kind = "semantic"
# xfail = "textwrap.TextWrapper.fill is a silent stub under mamba — no wrap/indent applied (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.TextWrapper: TextWrapper applies initial_indent to the first wrapped line and subsequent_indent to the rest"""
import textwrap

wrapper = textwrap.TextWrapper(width=20, initial_indent=">> ", subsequent_indent="   ")
result = wrapper.fill("hello world this is a long sentence")
lines = result.split("\n")
assert len(lines) > 1, f"expected multiple wrapped lines, got {lines!r}"
assert lines[0].startswith(">> "), f"initial_indent = {lines[0]!r}"
assert lines[1].startswith("   "), f"subsequent_indent = {lines[1]!r}"
print("textwrapper_initial_and_subsequent_indent OK")

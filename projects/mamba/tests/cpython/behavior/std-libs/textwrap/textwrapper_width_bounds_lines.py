# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "textwrapper_width_bounds_lines"
# subject = "textwrap.TextWrapper"
# kind = "semantic"
# xfail = "textwrap.TextWrapper.wrap is a silent stub under mamba — does not split to width (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.TextWrapper: a TextWrapper(width=20).wrap returns a list of strings each within the configured width"""
import textwrap

wrapper = textwrap.TextWrapper(width=20)
assert wrapper.width == 20, f"TextWrapper.width = {wrapper.width!r}"
wrapped = wrapper.wrap("the quick brown fox jumps over the lazy dog")
assert isinstance(wrapped, list), f"TextWrapper.wrap type = {type(wrapped)!r}"
assert len(wrapped) > 1, f"expected multiple lines, got {wrapped!r}"
assert all(len(s) <= 20 for s in wrapped), f"all within width = {wrapped!r}"
print("textwrapper_width_bounds_lines OK")

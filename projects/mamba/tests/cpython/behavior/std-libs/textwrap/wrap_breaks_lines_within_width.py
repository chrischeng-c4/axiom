# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_breaks_lines_within_width"
# subject = "textwrap.wrap"
# kind = "semantic"
# xfail = "textwrap.wrap is a silent stub under mamba — does not split to width (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.wrap: wrap returns a list of strings each within width and splits a long word when break_long_words=True (default)"""
import textwrap

words = "ab longer_word cd"
wrapped = textwrap.wrap(words, width=10)
assert isinstance(wrapped, list), f"wrap type = {type(wrapped)!r}"
assert all(isinstance(s, str) for s in wrapped), "all strings"
# "longer_word" (11 chars) exceeds width and is split by default.
assert all(len(line) <= 10 for line in wrapped), f"lines within width = {wrapped!r}"
print("wrap_breaks_lines_within_width OK")

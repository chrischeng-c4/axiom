# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_break_long_words_false_keeps_word_intact"
# subject = "textwrap.wrap"
# kind = "semantic"
# xfail = "textwrap.wrap is a silent stub under mamba — does not split to width (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.wrap: with break_long_words=False, a word longer than width stays intact on its own line"""
import textwrap

nobreak = textwrap.wrap(
    "short superlongwordthatexceedswidth end", width=10, break_long_words=False
)
assert any("superlongwordthatexceedswidth" in line for line in nobreak), (
    f"long word preserved = {nobreak!r}"
)
# Also with the simpler input from the legacy monolith.
nobreak2 = textwrap.wrap("ab longer_word cd", width=10, break_long_words=False)
assert any("longer_word" in line for line in nobreak2), f"long word intact = {nobreak2!r}"
print("wrap_break_long_words_false_keeps_word_intact OK")

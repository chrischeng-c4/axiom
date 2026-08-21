# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "fill_short_text_no_wrap"
# subject = "textwrap.fill"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.fill: fill on text shorter than width returns it unchanged with no newline; fill on the empty string returns the empty string"""
import textwrap

assert textwrap.fill("one two three", width=100) == "one two three", "short text no wrap"
assert textwrap.fill("") == "", "fill empty = ''"
print("fill_short_text_no_wrap OK")

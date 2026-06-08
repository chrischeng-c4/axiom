# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "word_boundary_zero_width_count"
# subject = "re.findall"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.findall: \\b is zero-width with one boundary on each side of a word: findall(r'\\b','a') is two hits, findall(r'\\B','a') is zero, and search(r'\\b','') is None"""
import re

assert len(re.findall(r"\b", "a")) == 2, "two boundaries around a single word"
assert len(re.findall(r"\B", "a")) == 0, "no non-boundary inside single char"
assert re.search(r"\b", "") is None, "no boundary in empty string"

print("word_boundary_zero_width_count OK")

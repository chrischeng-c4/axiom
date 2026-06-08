# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "word_boundary_anchor"
# subject = "re.search"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.search: \\b is a zero-width word boundary: r'\\b\\w+\\b' on '   hello   ' captures 'hello'"""
import re

m = re.search(r"\b\w+\b", "   hello   ")
assert m is not None and m.group() == "hello", f"word boundary group = {m.group() if m else None!r}"

print("word_boundary_anchor OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "search_reports_span_positions"
# subject = "re.search"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.search: the Match object exposes start/end/span: re.search(r'\\d+', 'abc123') gives start 3, end 6, span (3, 6)"""
import re

m = re.search(r"\d+", "abc123")
assert m is not None, "search found"
assert m.start() == 3, f"start = {m.start()!r}"
assert m.end() == 6, f"end = {m.end()!r}"
assert m.span() == (3, 6), f"span = {m.span()!r}"

print("search_reports_span_positions OK")

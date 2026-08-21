# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "span_start_end_per_group"
# subject = "re.Match.span"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Match.span: span(n)/start(n)/end(n) report per-group offsets: re.search(r'(\\d+)-(\\d+)','id: 12-345') has span()=(4,10), span(1)=(4,6), span(2)=(7,10)"""
import re

m = re.search(r"(\d+)-(\d+)", "id: 12-345")
assert m.span() == (4, 10), f"span() = {m.span()!r}"
assert m.span(1) == (4, 6), f"span(1) = {m.span(1)!r}"
assert m.span(2) == (7, 10), f"span(2) = {m.span(2)!r}"
assert m.start(2) == 7 and m.end(2) == 10, "start/end per group"

print("span_start_end_per_group OK")

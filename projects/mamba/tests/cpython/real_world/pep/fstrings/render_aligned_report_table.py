# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "real_world"
# case = "render_aligned_report_table"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: render an aligned plain-text report row using f-string width/alignment/precision specs over a small dataset, producing a fixed-width table"""
# f-strings are the idiomatic way to render aligned tabular text

rows = [
    ("alice", 7, 91.5),
    ("bob", 142, 8.0),
    ("carol", 3, 100.0),
]

lines = []
for name, count, pct in rows:
    lines.append(f"{name:<8}{count:>5}{pct:>8.2f}")

expected = [
    "alice       7   91.50",
    "bob       142    8.00",
    "carol       3  100.00",
]
assert lines == expected, f"table = {lines!r}"

print("render_aligned_report_table OK")

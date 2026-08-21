# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "delimiter_changes_separator_both_ways"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: a custom delimiter is used by writer output and honored by reader parsing"""
import csv
import io

buf = io.StringIO()
csv.writer(buf, delimiter="|").writerow(["x", "y", "z"])
out = buf.getvalue().strip()
assert out == "x|y|z", f"pipe delimiter = {out!r}"

rows = list(csv.reader(io.StringIO("x|y|z"), delimiter="|"))
assert rows == [["x", "y", "z"]], f"pipe reader = {rows!r}"

print("delimiter_changes_separator_both_ways OK")

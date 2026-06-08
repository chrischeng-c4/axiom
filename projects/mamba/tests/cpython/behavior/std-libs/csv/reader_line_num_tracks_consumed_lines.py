# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "reader_line_num_tracks_consumed_lines"
# subject = "csv.reader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.reader: reader.line_num starts at 0, increments per consumed source line, and stays put at EOF"""
import csv

reader = csv.reader(["line,1", "line,2", "line,3"])
assert reader.line_num == 0, f"start line_num = {reader.line_num}"
for expected in (1, 2, 3):
    next(reader)
    assert reader.line_num == expected, f"line_num = {reader.line_num}"

_stopped = False
try:
    next(reader)
except StopIteration:
    _stopped = True
assert _stopped, "expected StopIteration"
assert reader.line_num == 3, f"line_num after EOF = {reader.line_num}"

print("reader_line_num_tracks_consumed_lines OK")

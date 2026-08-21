# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "empty_field_round_trips"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: an empty middle field survives a writer -> reader round trip as an empty string"""
import csv
import io

buf = io.StringIO()
csv.writer(buf).writerow(["a", "", "c"])
buf.seek(0)
rows = list(csv.reader(buf))
assert rows == [["a", "", "c"]], f"empty field = {rows!r}"

print("empty_field_round_trips OK")

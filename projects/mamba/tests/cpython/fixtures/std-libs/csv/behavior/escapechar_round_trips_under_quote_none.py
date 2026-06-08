# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "escapechar_round_trips_under_quote_none"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: under QUOTE_NONE the escapechar escapes an embedded delimiter and the value round-trips through reader"""
import csv
import io

buf = io.StringIO()
csv.writer(buf, escapechar="\\", quoting=csv.QUOTE_NONE).writerow(["a,b", "c"])
assert buf.getvalue() == "a\\,b,c\r\n", f"escape write = {buf.getvalue()!r}"

buf.seek(0)
rows = list(csv.reader(buf, escapechar="\\", quoting=csv.QUOTE_NONE))
assert rows == [["a,b", "c"]], f"escape read = {rows!r}"

print("escapechar_round_trips_under_quote_none OK")

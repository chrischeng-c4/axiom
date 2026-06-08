# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "default_lineterminator_is_crlf"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: the default line terminator written after a row is carriage-return + newline"""
import csv
import io

buf = io.StringIO()
csv.writer(buf).writerow(["x"])
raw = buf.getvalue()
assert raw == "x\r\n", f"default line terminator = {raw!r}"

print("default_lineterminator_is_crlf OK")

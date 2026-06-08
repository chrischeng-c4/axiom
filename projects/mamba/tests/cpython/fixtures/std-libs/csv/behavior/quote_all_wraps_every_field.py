# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "quote_all_wraps_every_field"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: quoting=QUOTE_ALL wraps each written field in the quotechar"""
import csv
import io

buf = io.StringIO()
csv.writer(buf, quoting=csv.QUOTE_ALL).writerow(["a", "b", "c"])
out = buf.getvalue().strip()
assert out == '"a","b","c"', f"QUOTE_ALL = {out!r}"

print("quote_all_wraps_every_field OK")

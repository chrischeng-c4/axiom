# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "dictwriter_writeheader_returns_char_count"
# subject = "csv.DictWriter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.DictWriter: writeheader returns the number of characters written and emits the CRLF-terminated header"""
import csv
import io

buf = io.StringIO()
dw = csv.DictWriter(buf, fieldnames=["f1", "f2", "f3"])
assert dw.writeheader() == 10, "writeheader returns chars written"
assert buf.getvalue() == "f1,f2,f3\r\n", f"header = {buf.getvalue()!r}"

print("dictwriter_writeheader_returns_char_count OK")

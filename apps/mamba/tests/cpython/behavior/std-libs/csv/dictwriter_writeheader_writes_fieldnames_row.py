# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "dictwriter_writeheader_writes_fieldnames_row"
# subject = "csv.DictWriter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.DictWriter: DictWriter.writeheader emits the fieldnames as the first row, followed by mapped data rows"""
import csv
import io

buf = io.StringIO()
dw = csv.DictWriter(buf, fieldnames=["col1", "col2"])
dw.writeheader()
dw.writerow({"col1": "v1", "col2": "v2"})
buf.seek(0)
lines = buf.readlines()
assert lines[0].strip() == "col1,col2", f"header line = {lines[0]!r}"
assert lines[1].strip() == "v1,v2", f"data line = {lines[1]!r}"

print("dictwriter_writeheader_writes_fieldnames_row OK")

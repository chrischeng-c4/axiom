# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "writerows_emits_each_row"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: writerows emits one parsed row per input sequence in order"""
import csv
import io

buf = io.StringIO()
csv.writer(buf).writerows([["a", "b"], ["c", "d"], ["e", "f"]])
buf.seek(0)
rows = list(csv.reader(buf))
assert len(rows) == 3, f"writerows count = {len(rows)!r}"
assert rows[2] == ["e", "f"], f"row 2 = {rows[2]!r}"

print("writerows_emits_each_row OK")

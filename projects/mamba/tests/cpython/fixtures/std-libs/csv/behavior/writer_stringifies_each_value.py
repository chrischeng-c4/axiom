# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "writer_stringifies_each_value"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: writerow renders int/float/bool/None via str(): None becomes empty, others their str() form"""
import csv
import io

buf = io.StringIO()
csv.writer(buf).writerow([1, 2.5, True, None])
out = buf.getvalue().strip()
assert out == "1,2.5,True,", f"number writing = {out!r}"

print("writer_stringifies_each_value OK")

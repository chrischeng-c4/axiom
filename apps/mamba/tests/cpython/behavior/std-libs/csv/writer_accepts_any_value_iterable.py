# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "writer_accepts_any_value_iterable"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: writerow consumes an arbitrary iterable (e.g. a generator) and stringifies each value"""
import csv
import io

buf = io.StringIO()
csv.writer(buf).writerow((i * i for i in range(4)))
assert buf.getvalue() == "0,1,4,9\r\n", f"generator row = {buf.getvalue()!r}"

print("writer_accepts_any_value_iterable OK")

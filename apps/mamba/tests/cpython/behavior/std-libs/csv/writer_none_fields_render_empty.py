# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "writer_none_fields_render_empty"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: None fields render as empty; a lone None field becomes a quoted empty string"""
import csv
import io

buf = io.StringIO()
csv.writer(buf).writerows([["a", None], [None, "d"]])
assert buf.getvalue() == "a,\r\n,d\r\n", f"none mix = {buf.getvalue()!r}"

buf2 = io.StringIO()
csv.writer(buf2).writerows([[None], ["a"]])
assert buf2.getvalue() == '""\r\na\r\n', f"lone none = {buf2.getvalue()!r}"

print("writer_none_fields_render_empty OK")

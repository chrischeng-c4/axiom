# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "reader_quoted_field_keeps_embedded_comma"
# subject = "csv.reader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.reader: a double-quoted field preserves an embedded comma; remaining fields split normally and all come back as str"""
import csv
import io

rows = list(csv.reader(io.StringIO('"hello, world",42,True')))
assert rows == [["hello, world", "42", "True"]], f"quoted comma = {rows!r}"

print("reader_quoted_field_keeps_embedded_comma OK")

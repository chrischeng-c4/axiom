# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "dictreader_infers_fieldnames_from_first_row"
# subject = "csv.DictReader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.DictReader: DictReader uses the first row as fieldnames and maps each later row to those keys"""
import csv
import io

data = "id,name,score\n1,Alice,95\n2,Bob,87\n"
dr = csv.DictReader(io.StringIO(data))
assert dr.fieldnames == ["id", "name", "score"], f"fieldnames = {dr.fieldnames!r}"
rows = list(dr)
assert rows[0]["name"] == "Alice", f"name = {rows[0]['name']!r}"
assert rows[1]["score"] == "87", f"score = {rows[1]['score']!r}"

print("dictreader_infers_fieldnames_from_first_row OK")

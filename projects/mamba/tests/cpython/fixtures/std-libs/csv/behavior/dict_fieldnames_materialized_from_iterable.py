# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "dict_fieldnames_materialized_from_iterable"
# subject = "csv.DictReader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.DictReader: DictReader and DictWriter accept any iterable for fieldnames and store it as a list"""
import csv
import io

dr = csv.DictReader(io.StringIO("1,2\n"), fieldnames=iter(["a", "b"]))
assert dr.fieldnames == ["a", "b"], f"dr iter = {dr.fieldnames!r}"

dw = csv.DictWriter(io.StringIO(), iter(["a", "b", "c"]))
assert dw.fieldnames == ["a", "b", "c"], f"dw iter = {dw.fieldnames!r}"
assert isinstance(dw.fieldnames, list), "fieldnames stored as list"

print("dict_fieldnames_materialized_from_iterable OK")

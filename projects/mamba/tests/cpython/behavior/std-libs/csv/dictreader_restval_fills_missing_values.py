# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "dictreader_restval_fills_missing_values"
# subject = "csv.DictReader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.DictReader: missing trailing values are filled with restval; the default restval is None"""
import csv

r2 = csv.DictReader(["a,b\r\n"], fieldnames=["x", "y", "z"], restval="DEFAULT")
assert next(r2) == {"x": "a", "y": "b", "z": "DEFAULT"}, "restval"

r3 = csv.DictReader(["a\r\n"], fieldnames=["x", "y"])
assert next(r3) == {"x": "a", "y": None}, "default restval is None"

print("dictreader_restval_fills_missing_values OK")

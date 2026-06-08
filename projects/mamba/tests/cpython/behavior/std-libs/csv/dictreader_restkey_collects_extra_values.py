# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "dictreader_restkey_collects_extra_values"
# subject = "csv.DictReader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.DictReader: trailing values beyond fieldnames are collected into a list under restkey"""
import csv

reader = csv.DictReader(
    ["1,2,abc,4,5,6\r\n"], fieldnames=["f1", "f2"], restkey="_rest"
)
assert next(reader) == {"f1": "1", "f2": "2", "_rest": ["abc", "4", "5", "6"]}, "restkey"

print("dictreader_restkey_collects_extra_values OK")

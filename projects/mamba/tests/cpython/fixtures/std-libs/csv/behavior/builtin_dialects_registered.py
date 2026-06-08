# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "builtin_dialects_registered"
# subject = "csv.list_dialects"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.list_dialects: the built-in excel, excel-tab, and unix dialects are always present in list_dialects"""
import csv

builtins = csv.list_dialects()
assert "excel" in builtins, f"excel missing from {builtins!r}"
assert "excel-tab" in builtins, f"excel-tab missing from {builtins!r}"
assert "unix" in builtins, f"unix missing from {builtins!r}"

print("builtin_dialects_registered OK")

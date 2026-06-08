# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "custom_dialect_subclass_instance_accepted"
# subject = "csv.Dialect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.Dialect: a custom Dialect subclass instance drives parsing: adjacent delimiters yield empty fields"""
import csv


class SpaceDialect(csv.excel):
    delimiter = " "
    quoting = csv.QUOTE_NONE
    escapechar = "\\"


space = list(csv.reader(["abc   def", "one two"], dialect=SpaceDialect()))
# Adjacent delimiters produce empty fields (skipinitialspace is False).
assert space[0] == ["abc", "", "", "def"], f"space row 0 = {space[0]!r}"
assert space[1] == ["one", "two"], f"space row 1 = {space[1]!r}"

print("custom_dialect_subclass_instance_accepted OK")

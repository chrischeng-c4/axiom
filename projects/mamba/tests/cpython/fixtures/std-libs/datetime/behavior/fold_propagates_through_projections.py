# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "fold_propagates_through_projections"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime: fold propagates through .time() and .timetz() projections of a datetime built with fold=1"""
import datetime

dtf = datetime.datetime(1, 1, 1, fold=1)
assert dtf.time().fold == 1, "fold via time()"
assert dtf.timetz().fold == 1, "fold via timetz()"
print("fold_propagates_through_projections OK")

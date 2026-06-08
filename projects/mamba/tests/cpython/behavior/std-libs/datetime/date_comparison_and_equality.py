# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "date_comparison_and_equality"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: dates order chronologically (<) and compare by value: equal dates are ==, different days are !="""
import datetime

assert datetime.date(2023, 1, 1) < datetime.date(2023, 12, 31), "date <"
assert datetime.date(2023, 6, 15) == datetime.date(2023, 6, 15), "date =="
assert datetime.date(2023, 6, 15) != datetime.date(2023, 6, 16), "date !="
print("date_comparison_and_equality OK")

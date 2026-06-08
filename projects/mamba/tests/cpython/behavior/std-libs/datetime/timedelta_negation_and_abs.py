# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timedelta_negation_and_abs"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.timedelta: timedelta(days=-1).days == -1 and abs() of it has days == 1"""
import datetime

td_neg = datetime.timedelta(days=-1)
assert td_neg.days == -1, f"negative td days = {td_neg.days!r}"
td_abs = abs(td_neg)
assert td_abs.days == 1, f"abs(neg td) = {td_abs.days!r}"
print("timedelta_negation_and_abs OK")

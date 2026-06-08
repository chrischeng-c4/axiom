# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "fold_in_repr_only_when_set"
# subject = "datetime.time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.time: fold appears in repr only when set to 1: repr(time(fold=1)) and repr(datetime(...,fold=1)) include 'fold=1'"""
import datetime

assert repr(datetime.time(fold=1)) == "datetime.time(0, 0, fold=1)", \
    f"time fold repr = {repr(datetime.time(fold=1))!r}"
assert repr(datetime.datetime(1, 1, 1, fold=1)) == \
    "datetime.datetime(1, 1, 1, 0, 0, fold=1)", "datetime fold repr"
print("fold_in_repr_only_when_set OK")

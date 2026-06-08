# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "fold_does_not_affect_hash"
# subject = "datetime.time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.time: fold does not affect hashing: hash(t) == hash(t.replace(fold=1)) for both time and datetime"""
import datetime

t = datetime.time(0)
assert hash(t) == hash(t.replace(fold=1)), "time fold hash stable"
dt = datetime.datetime(1, 1, 1)
assert hash(dt) == hash(dt.replace(fold=1)), "datetime fold hash stable"
print("fold_does_not_affect_hash OK")

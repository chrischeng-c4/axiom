# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "ctime_format"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime: datetime(2002,3,2).ctime() == 'Sat Mar  2 00:00:00 2002' (note the space-padded day)"""
import datetime

assert datetime.datetime(2002, 3, 2).ctime() == "Sat Mar  2 00:00:00 2002", "ctime"
print("ctime_format OK")

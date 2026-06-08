# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "datetime_timedelta_instant_math"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime: adding/subtracting timedeltas lands on exact instants: a+hour==hour+a, a-hour==a+-hour, a+week, and a-(a+week)==-week"""
import datetime

a = datetime.datetime(2002, 3, 2, 17, 6)
hour = datetime.timedelta(0, 3600)
assert a + hour == datetime.datetime(2002, 3, 2, 18, 6), "a + hour"
assert hour + a == datetime.datetime(2002, 3, 2, 18, 6), "hour + a commutes"
assert a - hour == a + -hour, "subtract == add negative"
assert a + datetime.timedelta(7) == datetime.datetime(2002, 3, 9, 17, 6), "a + week"
assert a - (a + datetime.timedelta(7)) == -datetime.timedelta(7), "datetime difference"
print("datetime_timedelta_instant_math OK")

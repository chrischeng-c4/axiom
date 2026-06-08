# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timedelta_divmod_and_mod"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timedelta: divmod(td, unit) returns (int quotient, timedelta remainder), flooring toward -inf for a negative dividend, and % mirrors the remainder"""
import datetime

minute = datetime.timedelta(minutes=1)

t = datetime.timedelta(minutes=2, seconds=30)
q, r = divmod(t, minute)
assert q == 2, f"divmod q = {q!r}"
assert r == datetime.timedelta(seconds=30), f"divmod r = {r!r}"

# Negative dividend: quotient floors toward negative infinity.
tn = datetime.timedelta(minutes=-2, seconds=30)
q, r = divmod(tn, minute)
assert q == -2, f"neg divmod q = {q!r}"
assert r == datetime.timedelta(seconds=30), f"neg divmod r = {r!r}"

# Modulo mirrors divmod's remainder.
assert tn % minute == datetime.timedelta(seconds=30), f"mod = {tn % minute!r}"
print("timedelta_divmod_and_mod OK")

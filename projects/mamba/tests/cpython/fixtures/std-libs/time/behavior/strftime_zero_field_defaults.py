# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "strftime_zero_field_defaults"
# subject = "time.strftime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.strftime: strftime substitutes documented defaults for zero-valued fields: (2000,)+(0,)*8 with '%Y %m %d %H %M %S %w %j' yields '2000 01 01 00 00 00 1 001'"""
import time

# (2000,) + nine zeros -> year 2000, Jan 1, weekday Sat (%w=1), yday 001.
_zero = time.strftime("%Y %m %d %H %M %S %w %j", (2000,) + (0,) * 8)
assert _zero == "2000 01 01 00 00 00 1 001", f"zero-default strftime = {_zero!r}"
print("strftime_zero_field_defaults OK")

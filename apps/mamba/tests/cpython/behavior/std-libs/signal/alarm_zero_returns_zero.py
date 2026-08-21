# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "alarm_zero_returns_zero"
# subject = "signal.alarm"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.alarm: alarm(0) cancels any pending alarm and returns 0 (no previous alarm was scheduled)"""
import signal

# No alarm has been scheduled, so cancelling returns the 0-second remainder.
assert signal.alarm(0) == 0, "alarm(0) returns 0 when none scheduled"
print("alarm_zero_returns_zero OK")

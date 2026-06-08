# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "alarm_is_callable"
# subject = "signal.alarm"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.alarm: alarm_is_callable (surface)."""
import signal

assert callable(signal.alarm)
print("alarm_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_alarm_is_present"
# subject = "signal.alarm"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.alarm: api_alarm_is_present (surface)."""
import signal

assert hasattr(signal, "alarm")
print("api_alarm_is_present OK")

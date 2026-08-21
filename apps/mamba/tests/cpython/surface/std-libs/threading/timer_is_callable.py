# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "timer_is_callable"
# subject = "threading.Timer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Timer: timer_is_callable (surface)."""
import threading

assert callable(threading.Timer)
print("timer_is_callable OK")

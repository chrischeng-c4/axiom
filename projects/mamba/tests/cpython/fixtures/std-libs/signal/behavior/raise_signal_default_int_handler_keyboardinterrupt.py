# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "raise_signal_default_int_handler_keyboardinterrupt"
# subject = "signal.raise_signal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.raise_signal: with default_int_handler installed for SIGINT, raise_signal(SIGINT) surfaces synchronously as KeyboardInterrupt"""
import signal

signal.signal(signal.SIGINT, signal.default_int_handler)
hit_kbd = False
try:
    signal.raise_signal(signal.SIGINT)
except KeyboardInterrupt:
    hit_kbd = True
assert hit_kbd, "raise_signal(SIGINT) raises KeyboardInterrupt"

signal.signal(signal.SIGINT, signal.default_int_handler)
print("raise_signal_default_int_handler_keyboardinterrupt OK")

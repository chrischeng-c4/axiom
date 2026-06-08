# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "event_clear_resets_is_set"
# subject = "threading.Event"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Event: Event.set() makes is_set() true; Event.clear() resets is_set() to false"""
import threading

_ev = threading.Event()
_ev.set()
assert _ev.is_set(), "event set after set()"
_ev.clear()
assert not _ev.is_set(), "event unset after clear()"

print("event_clear_resets_is_set OK")

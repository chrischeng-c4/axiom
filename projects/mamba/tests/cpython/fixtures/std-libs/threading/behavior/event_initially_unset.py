# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "event_initially_unset"
# subject = "threading.Event"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Event: a freshly constructed Event reports is_set() == False"""
import threading

_ev = threading.Event()
assert not _ev.is_set(), "a fresh Event is initially unset"

print("event_initially_unset OK")

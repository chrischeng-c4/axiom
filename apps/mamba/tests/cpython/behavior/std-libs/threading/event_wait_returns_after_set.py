# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "event_wait_returns_after_set"
# subject = "threading.Event"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Event: Event.wait() in a worker returns only after the main thread calls set()"""
import threading

import time

_ev = threading.Event()
_ev_result = []

def _waiter():
    _ev.wait()
    _ev_result.append("done")

tw = threading.Thread(target=_waiter)
tw.start()
time.sleep(0.01)
_ev.set()
tw.join()
assert _ev_result == ["done"], f"event result = {_ev_result!r}"

print("event_wait_returns_after_set OK")

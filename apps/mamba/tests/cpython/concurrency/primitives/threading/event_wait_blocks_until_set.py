# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "primitives"
# lib = "threading"
# dimension = "concurrency"
# case = "event_wait_blocks_until_set"
# subject = "threading.Event.wait"
# kind = "semantic"
# xfail = "issue #3128: Event.wait returns is_set() immediately instead of blocking"
# mem_carveout = ""
# source = "issue #3128 negative-control probe"
# status = "filled"
# ///
"""Concurrency primitive: Event.wait must block until another thread sets it.

The elapsed time is the assertion. A stub that returns `is_set()` without
waiting produces the *correct return value* in the unset case, so a
return-value-only check cannot tell a real wait from an immediate return:

* PASS: wait() returns True only after the setter fires, and an unset
  wait(timeout) burns its full timeout before returning False
* FAIL: either wait returns before its cause, or the values are wrong
"""
import threading
import time

DELAY = 0.4
TIMEOUT = 0.5
problems: list[str] = []

# --- positive: wait() unblocks when another thread sets the event ---
event = threading.Event()


def setter() -> None:
    time.sleep(DELAY)
    event.set()


worker = threading.Thread(target=setter)
started = time.monotonic()
worker.start()
returned = event.wait(timeout=5.0)
waited = time.monotonic() - started
worker.join()

if returned is not True:
    problems.append(f"wait() returned {returned!r}, expected True")
if waited < DELAY * 0.7:
    problems.append(f"wait() returned after {waited:.3f}s, expected >= {DELAY * 0.7:.3f}s")

# --- negative control: an event nobody sets must burn the whole timeout ---
never = threading.Event()
started = time.monotonic()
timed_out = never.wait(timeout=TIMEOUT)
burned = time.monotonic() - started

if timed_out is not False:
    problems.append(f"unset wait(timeout) returned {timed_out!r}, expected False")
if burned < TIMEOUT * 0.7:
    problems.append(f"unset wait(timeout) returned after {burned:.3f}s, expected >= {TIMEOUT * 0.7:.3f}s")

if problems:
    print(f"concurrency: FAIL: {problems[0]}")
else:
    print("concurrency: PASS")

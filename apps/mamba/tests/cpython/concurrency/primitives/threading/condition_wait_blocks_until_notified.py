# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "primitives"
# lib = "threading"
# dimension = "concurrency"
# case = "condition_wait_blocks_until_notified"
# subject = "threading.Condition.wait"
# kind = "semantic"
# xfail = "issue #3128: Condition.wait returns at once; there is no waiter queue"
# mem_carveout = ""
# source = "issue #3128 negative-control probe"
# status = "filled"
# ///
"""Concurrency primitive: Condition.wait blocks until notify(), then reacquires.

A stub that returns True without waiting produces the same *value* as a real
notified wait, so the elapsed time carries the assertion. The unnotified case is
the negative control: it must burn its whole timeout and return False.

* PASS: notified wait returns True only after the notifier fires; unnotified
  wait returns False only after its timeout elapses
* FAIL: either wait returns before its cause, or the values are wrong
"""
import threading
import time

DELAY = 0.4
TIMEOUT = 0.4
problems: list[str] = []

condition = threading.Condition()
delivered: list[bool] = [False]


def notifier() -> None:
    time.sleep(DELAY)
    with condition:
        delivered[0] = True
        condition.notify()


worker = threading.Thread(target=notifier)
started = time.monotonic()
worker.start()
with condition:
    notified = condition.wait(timeout=5.0)
waited = time.monotonic() - started
worker.join()

if notified is not True:
    problems.append(f"notified wait() returned {notified!r}, expected True")
if waited < DELAY * 0.7:
    problems.append(f"notified wait() returned after {waited:.3f}s, expected >= {DELAY * 0.7:.3f}s")
if not delivered[0]:
    problems.append("wait() returned before the notifier ran")

# --- negative control: nobody notifies, so the timeout must be burned ---
lonely = threading.Condition()
started = time.monotonic()
with lonely:
    timed_out = lonely.wait(timeout=TIMEOUT)
burned = time.monotonic() - started

if timed_out is not False:
    problems.append(f"unnotified wait(timeout) returned {timed_out!r}, expected False")
if burned < TIMEOUT * 0.7:
    problems.append(f"unnotified wait(timeout) returned after {burned:.3f}s, expected >= {TIMEOUT * 0.7:.3f}s")

if problems:
    print(f"concurrency: FAIL: {problems[0]}")
else:
    print("concurrency: PASS")

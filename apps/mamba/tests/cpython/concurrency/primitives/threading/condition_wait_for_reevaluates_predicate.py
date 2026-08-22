# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "primitives"
# lib = "threading"
# dimension = "concurrency"
# case = "condition_wait_for_reevaluates_predicate"
# subject = "threading.Condition.wait_for"
# kind = "semantic"
# xfail = "issue #3128: wait_for evaluates the predicate once and returns immediately"
# mem_carveout = ""
# source = "issue #3128 negative-control probe"
# status = "filled"
# ///
"""Concurrency primitive: Condition.wait_for re-evaluates its predicate.

`wait_for` is a loop, not a single test: it must call the predicate, block on
notify, and call it again. Counting the calls is what separates a real loop from
a single evaluation that happens to return the right value later.

* PASS: predicate seen False first and True after the flip, called more than
  once, and wait_for returns True only after the flipper ran
* FAIL: one evaluation, an immediate return, or a wrong value
"""
import threading
import time

DELAY = 0.4
problems: list[str] = []

condition = threading.Condition()
flag: list[bool] = [False]
calls: list[int] = [0]


def predicate() -> bool:
    calls[0] += 1
    return flag[0]


def flipper() -> None:
    time.sleep(DELAY)
    with condition:
        flag[0] = True
        condition.notify_all()


worker = threading.Thread(target=flipper)
started = time.monotonic()
worker.start()
with condition:
    satisfied = condition.wait_for(predicate, timeout=5.0)
waited = time.monotonic() - started
worker.join()

if satisfied is not True:
    problems.append(f"wait_for returned {satisfied!r}, expected True")
if waited < DELAY * 0.7:
    problems.append(f"wait_for returned after {waited:.3f}s, expected >= {DELAY * 0.7:.3f}s")
if calls[0] < 2:
    problems.append(f"predicate called {calls[0]} time(s), expected re-evaluation after notify")

# --- negative control: a predicate that never becomes true must time out ---
stuck = threading.Condition()
never_calls: list[int] = [0]


def never() -> bool:
    never_calls[0] += 1
    return False


started = time.monotonic()
with stuck:
    gave_up = stuck.wait_for(never, timeout=0.4)
burned = time.monotonic() - started

if gave_up is not False:
    problems.append(f"unsatisfiable wait_for returned {gave_up!r}, expected False")
if burned < 0.28:
    problems.append(f"unsatisfiable wait_for returned after {burned:.3f}s, expected >= 0.280s")

if problems:
    print(f"concurrency: FAIL: {problems[0]}")
else:
    print("concurrency: PASS")

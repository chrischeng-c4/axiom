# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "primitives"
# lib = "threading"
# dimension = "concurrency"
# case = "semaphore_caps_concurrent_holders"
# subject = "threading.Semaphore.acquire"
# kind = "semantic"
# xfail = "issue #3128: a zero Semaphore cannot block, so the permit count caps nothing"
# mem_carveout = ""
# source = "issue #3128 negative-control probe"
# status = "filled"
# ///
"""Concurrency primitive: Semaphore(N) admits at most N holders at once.

Six threads contend for two permits. The peak simultaneous-holder count is the
assertion — a semaphore whose acquire never blocks lets all six in at once and
still exits cleanly, so only the peak reveals it.

* PASS: peak holders == 2
* FAIL: any other peak, or a permit-exhausted acquire that does not block
"""
import threading
import time

PERMITS = 2
WORKERS = 6
HOLD = 0.2
problems: list[str] = []

semaphore = threading.Semaphore(PERMITS)
guard = threading.Lock()
live: list[int] = [0]
peak: list[int] = [0]


def hold_a_permit() -> None:
    semaphore.acquire()
    with guard:
        live[0] += 1
        if live[0] > peak[0]:
            peak[0] = live[0]
    time.sleep(HOLD)
    with guard:
        live[0] -= 1
    semaphore.release()


threads = [threading.Thread(target=hold_a_permit) for _ in range(WORKERS)]
for thread in threads:
    thread.start()
for thread in threads:
    thread.join()

if peak[0] != PERMITS:
    problems.append(f"peak concurrent holders {peak[0]}, expected {PERMITS}")
if live[0] != 0:
    problems.append(f"{live[0]} holders still counted after join")

# --- negative control: an exhausted semaphore must block until released ---
drained = threading.Semaphore(0)


def release_late() -> None:
    time.sleep(HOLD)
    drained.release()


releaser = threading.Thread(target=release_late)
started = time.monotonic()
releaser.start()
got = drained.acquire(timeout=5.0)
waited = time.monotonic() - started
releaser.join()

if got is not True:
    problems.append(f"exhausted acquire returned {got!r}, expected True")
if waited < HOLD * 0.7:
    problems.append(f"exhausted acquire returned after {waited:.3f}s, expected >= {HOLD * 0.7:.3f}s")

# --- non-blocking acquire must stay non-blocking ---
empty = threading.Semaphore(0)
started = time.monotonic()
refused = empty.acquire(blocking=False)
instant = time.monotonic() - started

if refused is not False:
    problems.append(f"acquire(blocking=False) returned {refused!r}, expected False")
if instant > 0.2:
    problems.append(f"acquire(blocking=False) took {instant:.3f}s, expected to return at once")

if problems:
    print(f"concurrency: FAIL: {problems[0]}")
else:
    print("concurrency: PASS")

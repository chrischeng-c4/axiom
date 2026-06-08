# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "semaphore_limits_concurrency"
# subject = "threading.Semaphore"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Semaphore: Semaphore(2) grants two non-blocking acquires, refuses the third, and grants again after a release"""
import threading

_sem = threading.Semaphore(2)
assert _sem.acquire(blocking=False), "first acquire succeeds"
assert _sem.acquire(blocking=False), "second acquire succeeds"
assert not _sem.acquire(blocking=False), "third acquire fails (limit=2)"
_sem.release()
assert _sem.acquire(blocking=False), "after release, acquire succeeds again"
_sem.release()
_sem.release()

print("semaphore_limits_concurrency OK")

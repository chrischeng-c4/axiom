# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "lock_serializes_increments"
# subject = "threading.Lock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Lock: a shared Lock serializes 5 threads x 100 increments of a shared counter to exactly 500"""
import threading

_counter = [0]
_lock = threading.Lock()

def _increment():
    for _ in range(100):
        with _lock:
            _counter[0] += 1

_threads = [threading.Thread(target=_increment) for _ in range(5)]
for _th in _threads:
    _th.start()
for _th in _threads:
    _th.join()
assert _counter[0] == 500, f"locked counter = {_counter[0]!r}"

print("lock_serializes_increments OK")

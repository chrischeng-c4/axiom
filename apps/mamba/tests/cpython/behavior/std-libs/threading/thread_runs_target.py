# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "thread_runs_target"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: a Thread runs its target function and the side effect is visible after join()"""
import threading

_result = []

def _worker():
    _result.append(42)

t = threading.Thread(target=_worker)
t.start()
t.join()
assert _result == [42], f"thread result = {_result!r}"

print("thread_runs_target OK")

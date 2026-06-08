# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "run_one_shot_drain"
# subject = "atexit._run_exitfuncs"
# kind = "semantic"
# xfail = "_run_exitfuncs() never invokes registered handlers (stub, #652)"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit._run_exitfuncs: _run_exitfuncs() runs each callback exactly once; a second call is a no-op because the queue is already drained"""
import atexit

once = []


def mark():
    once.append(1)


atexit._clear()
atexit.register(mark)
atexit._run_exitfuncs()
atexit._run_exitfuncs()
assert once == [1], f"callback fires exactly once across two runs: {once}"
atexit._clear()
print("run_one_shot_drain OK")

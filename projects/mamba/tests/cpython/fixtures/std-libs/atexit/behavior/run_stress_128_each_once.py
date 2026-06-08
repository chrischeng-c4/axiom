# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "run_stress_128_each_once"
# subject = "atexit._run_exitfuncs"
# kind = "semantic"
# xfail = "_run_exitfuncs() never invokes registered handlers (stub, #652)"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit._run_exitfuncs: 128 registered callbacks each fire exactly once and the queue drains to empty"""
import atexit

counter = [0]


def bump():
    counter[0] += 1


atexit._clear()
for _ in range(128):
    atexit.register(bump)
atexit._run_exitfuncs()
assert counter[0] == 128, f"every callback fired once: {counter[0]}"
assert atexit._ncallbacks() == 0, "queue drained after run"
atexit._clear()
print("run_stress_128_each_once OK")

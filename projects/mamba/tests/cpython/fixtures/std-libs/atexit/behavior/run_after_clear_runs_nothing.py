# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "run_after_clear_runs_nothing"
# subject = "atexit._run_exitfuncs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit._run_exitfuncs: _run_exitfuncs() after _clear() runs nothing and leaves an empty queue"""
import atexit

fired = []


def cleanup():
    fired.append(1)


atexit._clear()
atexit.register(cleanup)
atexit._clear()
result = atexit._run_exitfuncs()
assert result is None, f"_run_exitfuncs() returns None: {result!r}"
assert fired == [], f"cleared callback must not fire: {fired}"
assert atexit._ncallbacks() == 0, "queue stays empty"
print("run_after_clear_runs_nothing OK")

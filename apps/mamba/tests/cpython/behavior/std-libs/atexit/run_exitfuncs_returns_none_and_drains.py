# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "run_exitfuncs_returns_none_and_drains"
# subject = "atexit._run_exitfuncs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit._run_exitfuncs: _run_exitfuncs() returns None and drains the queue so _ncallbacks() is 0 afterwards"""
import atexit


def cleanup():
    pass


atexit._clear()
atexit.register(cleanup)
assert atexit._ncallbacks() == 1, "one callback registered"
result = atexit._run_exitfuncs()
assert result is None, f"_run_exitfuncs() returns None: {result!r}"
assert atexit._ncallbacks() == 0, "queue drained after running"
atexit._clear()
print("run_exitfuncs_returns_none_and_drains OK")

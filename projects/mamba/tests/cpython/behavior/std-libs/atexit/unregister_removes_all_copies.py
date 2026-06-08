# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "unregister_removes_all_copies"
# subject = "atexit.unregister"
# kind = "semantic"
# xfail = "unregister matches by string name not callable identity, and _run_exitfuncs() never fires handlers (stub, #652)"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit.unregister: unregister() removes every copy of a duplicated registration so the callback fires zero times"""
import atexit

fired = []


def note():
    fired.append(1)


atexit._clear()
atexit.register(note)
atexit.register(note)
atexit.unregister(note)  # cancels BOTH copies
atexit._run_exitfuncs()
# The observable contract: every copy is cancelled, so the callback fires
# zero times.
assert fired == [], f"removed callback must not fire: {fired}"
atexit._clear()
print("unregister_removes_all_copies OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "unregister_missing_is_silent"
# subject = "atexit.unregister"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit.unregister: unregister() of a callable that was never registered is a silent no-op (no raise), queue length unchanged"""
import atexit


def never_registered():
    pass


atexit._clear()
# No raise even though `never_registered` is not in the queue.
atexit.unregister(never_registered)
assert atexit._ncallbacks() == 0, "queue length unchanged by no-op unregister"
atexit._clear()
print("unregister_missing_is_silent OK")

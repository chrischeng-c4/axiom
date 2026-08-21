# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "unregister_by_identity_net_effect"
# subject = "atexit.unregister"
# kind = "semantic"
# xfail = "unregister matches by string name not callable identity, and _run_exitfuncs() never fires handlers (stub, #652)"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit.unregister: unregister() matches by callable identity: dropping one of two distinct registered callables leaves only the other to fire"""
import atexit

a = [0]


def inc():
    a[0] += 1


def dec():
    a[0] -= 1


atexit._clear()
for _ in range(4):
    atexit.register(inc)
atexit.register(dec)
atexit.unregister(inc)  # drops all four inc registrations by identity
atexit._run_exitfuncs()
assert a[0] == -1, f"only dec survived: {a[0]}"
atexit._clear()
print("unregister_by_identity_net_effect OK")

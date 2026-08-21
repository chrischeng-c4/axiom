# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "register_bound_method_fires"
# subject = "atexit.register"
# kind = "semantic"
# xfail = "_run_exitfuncs() never invokes registered handlers (stub, #652)"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit.register: a bound method registered with an argument fires at _run_exitfuncs() like any other callable"""
import atexit

atexit._clear()
collected = []
atexit.register(collected.append, 5)
atexit._run_exitfuncs()
assert collected == [5], f"bound method fired with arg: {collected}"
atexit._clear()
print("register_bound_method_fires OK")

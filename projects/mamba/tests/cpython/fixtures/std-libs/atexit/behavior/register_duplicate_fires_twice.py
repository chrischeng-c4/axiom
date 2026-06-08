# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "register_duplicate_fires_twice"
# subject = "atexit.register"
# kind = "semantic"
# xfail = "_run_exitfuncs() never invokes registered handlers (stub, #652)"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit.register: the same callable registered twice fires twice at _run_exitfuncs()"""
import atexit

atexit._clear()
hits = []
atexit.register(hits.append, "x")
atexit.register(hits.append, "x")
atexit._run_exitfuncs()
assert hits == ["x", "x"], f"duplicate registration fires twice: {hits}"
atexit._clear()
print("register_duplicate_fires_twice OK")

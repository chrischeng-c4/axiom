# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "register_returns_registered_callable"
# subject = "atexit.register"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit.register: register() returns the callable it was given (identity), the documented return contract"""
import atexit


def cleanup():
    pass


atexit._clear()
ret = atexit.register(cleanup)
assert ret is cleanup, f"register should return the callable: {ret!r}"
atexit._clear()
print("register_returns_registered_callable OK")

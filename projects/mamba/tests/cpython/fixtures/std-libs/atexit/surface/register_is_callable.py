# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "surface"
# case = "register_is_callable"
# subject = "atexit.register"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""atexit.register: register_is_callable (surface)."""
import atexit

assert callable(atexit.register)
print("register_is_callable OK")

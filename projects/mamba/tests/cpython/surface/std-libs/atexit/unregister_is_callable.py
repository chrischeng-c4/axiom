# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "surface"
# case = "unregister_is_callable"
# subject = "atexit.unregister"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""atexit.unregister: unregister_is_callable (surface)."""
import atexit

assert callable(atexit.unregister)
print("unregister_is_callable OK")

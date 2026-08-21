# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "surface"
# case = "ncallbacks_is_callable"
# subject = "atexit._ncallbacks"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""atexit._ncallbacks: ncallbacks_is_callable (surface)."""
import atexit

assert callable(atexit._ncallbacks)
print("ncallbacks_is_callable OK")

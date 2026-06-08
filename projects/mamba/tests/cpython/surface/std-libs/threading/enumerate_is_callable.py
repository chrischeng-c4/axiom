# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "enumerate_is_callable"
# subject = "threading.enumerate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.enumerate: enumerate_is_callable (surface)."""
import threading

assert callable(threading.enumerate)
print("enumerate_is_callable OK")

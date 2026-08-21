# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "local_is_callable"
# subject = "threading.local"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.local: local_is_callable (surface)."""
import threading

assert callable(threading.local)
print("local_is_callable OK")

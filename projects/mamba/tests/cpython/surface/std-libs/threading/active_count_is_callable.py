# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "active_count_is_callable"
# subject = "threading.active_count"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.active_count: active_count_is_callable (surface)."""
import threading

assert callable(threading.active_count)
print("active_count_is_callable OK")

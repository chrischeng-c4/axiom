# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "condition_is_callable"
# subject = "threading.Condition"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Condition: condition_is_callable (surface)."""
import threading

assert callable(threading.Condition)
print("condition_is_callable OK")

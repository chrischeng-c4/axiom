# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "queue_is_callable"
# subject = "multiprocessing.Queue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing.Queue: queue_is_callable (surface)."""
import multiprocessing

assert callable(multiprocessing.Queue)
print("queue_is_callable OK")

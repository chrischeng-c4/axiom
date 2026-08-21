# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "full_is_exception_subclass"
# subject = "queue.Full"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""queue.Full: full_is_exception_subclass (surface)."""
import queue

assert hasattr(queue.Full, "__cause__")
print("full_is_exception_subclass OK")

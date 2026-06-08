# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "empty_is_exception_subclass"
# subject = "queue.Empty"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""queue.Empty: empty_is_exception_subclass (surface)."""
import queue

assert hasattr(queue.Empty, "__cause__")
print("empty_is_exception_subclass OK")

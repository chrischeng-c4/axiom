# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "empty_attr"
# subject = "queue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""queue: empty_attr (surface)."""
import queue

assert hasattr(queue, "Empty")
print("empty_attr OK")

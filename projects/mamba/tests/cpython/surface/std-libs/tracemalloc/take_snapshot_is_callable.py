# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "surface"
# case = "take_snapshot_is_callable"
# subject = "tracemalloc.take_snapshot"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tracemalloc.take_snapshot: take_snapshot_is_callable (surface)."""
import tracemalloc

assert callable(tracemalloc.take_snapshot)
print("take_snapshot_is_callable OK")

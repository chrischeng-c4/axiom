# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "surface"
# case = "snapshot_is_callable"
# subject = "tracemalloc.Snapshot"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tracemalloc.Snapshot: snapshot_is_callable (surface)."""
import tracemalloc

assert callable(tracemalloc.Snapshot)
print("snapshot_is_callable OK")

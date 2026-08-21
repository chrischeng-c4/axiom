# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "surface"
# case = "import_heapq"
# subject = "heapq"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq: import_heapq (surface)."""
import heapq

assert hasattr(heapq, "heappush")
print("import_heapq OK")

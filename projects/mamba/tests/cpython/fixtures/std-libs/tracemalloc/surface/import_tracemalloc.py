# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "surface"
# case = "import_tracemalloc"
# subject = "tracemalloc"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tracemalloc: import_tracemalloc (surface)."""
import tracemalloc

assert hasattr(tracemalloc, "start")
print("import_tracemalloc OK")

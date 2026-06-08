# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "import_multiprocessing"
# subject = "multiprocessing"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing: import_multiprocessing (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "Process")
print("import_multiprocessing OK")

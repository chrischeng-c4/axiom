# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "pipe_is_callable"
# subject = "multiprocessing.Pipe"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing.Pipe: pipe_is_callable (surface)."""
import multiprocessing

assert callable(multiprocessing.Pipe)
print("pipe_is_callable OK")

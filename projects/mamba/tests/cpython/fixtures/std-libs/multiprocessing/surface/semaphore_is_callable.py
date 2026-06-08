# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "semaphore_is_callable"
# subject = "multiprocessing.Semaphore"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing.Semaphore: semaphore_is_callable (surface)."""
import multiprocessing

assert callable(multiprocessing.Semaphore)
print("semaphore_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "active_children_is_callable"
# subject = "multiprocessing.active_children"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing.active_children: active_children_is_callable (surface)."""
import multiprocessing

assert callable(multiprocessing.active_children)
print("active_children_is_callable OK")

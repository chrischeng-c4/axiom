# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "getclasstree_is_callable"
# subject = "inspect.getclasstree"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getclasstree: getclasstree_is_callable (surface)."""
import inspect

assert callable(inspect.getclasstree)
print("getclasstree_is_callable OK")

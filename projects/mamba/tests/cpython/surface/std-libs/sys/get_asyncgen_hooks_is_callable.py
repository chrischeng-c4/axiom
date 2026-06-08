# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "get_asyncgen_hooks_is_callable"
# subject = "sys.get_asyncgen_hooks"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.get_asyncgen_hooks: get_asyncgen_hooks_is_callable (surface)."""
import sys

assert callable(sys.get_asyncgen_hooks)
print("get_asyncgen_hooks_is_callable OK")

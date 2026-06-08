# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "surface"
# case = "abstract_context_manager_attr"
# subject = "contextlib"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextlib: abstract_context_manager_attr (surface)."""
import contextlib

assert hasattr(contextlib, "AbstractContextManager")
print("abstract_context_manager_attr OK")

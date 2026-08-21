# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "surface"
# case = "util_find_spec_is_callable"
# subject = "importlib.util.find_spec"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""importlib.util.find_spec: util_find_spec_is_callable (surface)."""
import importlib.util

assert callable(importlib.util.find_spec)
print("util_find_spec_is_callable OK")

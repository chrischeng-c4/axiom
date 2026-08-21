# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "surface"
# case = "util_submodule_not_callable"
# subject = "importlib.util"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""importlib.util: util_submodule_not_callable (surface)."""
import importlib.util

assert not callable(importlib.util)
print("util_submodule_not_callable OK")

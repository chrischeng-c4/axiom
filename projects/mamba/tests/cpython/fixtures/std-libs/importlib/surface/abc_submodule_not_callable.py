# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "surface"
# case = "abc_submodule_not_callable"
# subject = "importlib.abc"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""importlib.abc: abc_submodule_not_callable (surface)."""
import importlib.abc

assert not callable(importlib.abc)
print("abc_submodule_not_callable OK")

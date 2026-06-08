# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "surface"
# case = "machinery_submodule_not_callable"
# subject = "importlib.machinery"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""importlib.machinery: machinery_submodule_not_callable (surface)."""
import importlib.machinery

assert not callable(importlib.machinery)
print("machinery_submodule_not_callable OK")

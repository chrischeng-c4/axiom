# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "surface"
# case = "import_module_is_callable"
# subject = "importlib.import_module"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""importlib.import_module: import_module_is_callable (surface)."""
import importlib

assert callable(importlib.import_module)
print("import_module_is_callable OK")

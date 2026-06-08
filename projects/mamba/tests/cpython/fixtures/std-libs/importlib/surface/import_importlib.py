# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "surface"
# case = "import_importlib"
# subject = "importlib"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""importlib: import_importlib (surface)."""
import importlib

assert hasattr(importlib, "import_module")
print("import_importlib OK")

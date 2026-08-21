# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "behavior"
# case = "import_module_imports_real_module"
# subject = "importlib.import_module"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_importlib"
# status = "filled"
# ///
"""importlib.import_module: import_module("json") returns the json module object whose __name__ is 'json', equivalent to a plain import"""
import importlib

mod = importlib.import_module("json")
assert mod.__name__ == "json", mod.__name__
import json
assert mod is json, "import_module returns the same module object as a plain import"
print("import_module_imports_real_module OK")

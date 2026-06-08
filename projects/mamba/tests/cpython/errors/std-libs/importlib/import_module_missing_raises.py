# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "errors"
# case = "import_module_missing_raises"
# subject = "importlib.import_module"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_importlib"
# status = "filled"
# ///
"""importlib.import_module: import_module_missing_raises (errors)."""
import importlib

_raised = False
try:
    importlib.import_module("no_such_module_xyzzy_123")
except ModuleNotFoundError:
    _raised = True
assert _raised, "import_module_missing_raises: expected ModuleNotFoundError"
print("import_module_missing_raises OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "errors"
# case = "import_module_empty_name_raises"
# subject = "importlib.import_module"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_importlib"
# status = "filled"
# ///
"""importlib.import_module: import_module_empty_name_raises (errors)."""
import importlib

_raised = False
try:
    importlib.import_module("")
except ValueError:
    _raised = True
assert _raised, "import_module_empty_name_raises: expected ValueError"
print("import_module_empty_name_raises OK")

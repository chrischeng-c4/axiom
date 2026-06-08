# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "errors"
# case = "import_module_relative_no_package_raises"
# subject = "importlib.import_module"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_importlib"
# status = "filled"
# ///
"""importlib.import_module: import_module_relative_no_package_raises (errors)."""
import importlib

_raised = False
try:
    importlib.import_module(".relative_no_pkg")
except TypeError:
    _raised = True
assert _raised, "import_module_relative_no_package_raises: expected TypeError"
print("import_module_relative_no_package_raises OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "behavior"
# case = "module_not_found_error_is_import_error"
# subject = "importlib.import_module"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_importlib"
# status = "filled"
# ///
"""importlib.import_module: ModuleNotFoundError is a subclass of ImportError, so import_module misses can be caught as ImportError"""
import importlib

assert issubclass(ModuleNotFoundError, ImportError), "ModuleNotFoundError <: ImportError"

caught_as_import_error = False
try:
    importlib.import_module("no_such_module_xyzzy_123")
except ImportError:
    caught_as_import_error = True
assert caught_as_import_error, "a missing import is catchable as ImportError"
print("module_not_found_error_is_import_error OK")

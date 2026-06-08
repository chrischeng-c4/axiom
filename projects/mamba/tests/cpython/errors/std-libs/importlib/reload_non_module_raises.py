# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "errors"
# case = "reload_non_module_raises"
# subject = "importlib.reload"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_importlib"
# status = "filled"
# ///
"""importlib.reload: reload_non_module_raises (errors)."""
import importlib

_raised = False
try:
    importlib.reload(42)
except TypeError:
    _raised = True
assert _raised, "reload_non_module_raises: expected TypeError"
print("reload_non_module_raises OK")

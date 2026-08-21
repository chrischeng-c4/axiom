# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "int_path_component_raises"
# subject = "pathlib.Path"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: int_path_component_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.Path(123)
except TypeError:
    _raised = True
assert _raised, "int_path_component_raises: expected TypeError"
print("int_path_component_raises OK")

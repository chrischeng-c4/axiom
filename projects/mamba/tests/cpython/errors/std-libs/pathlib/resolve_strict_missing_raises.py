# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "resolve_strict_missing_raises"
# subject = "pathlib.Path.resolve"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path.resolve: resolve_strict_missing_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.Path('/no/such/path/to/resolve_strict').resolve(strict=True)
except FileNotFoundError:
    _raised = True
assert _raised, "resolve_strict_missing_raises: expected FileNotFoundError"
print("resolve_strict_missing_raises OK")

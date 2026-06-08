# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "str_file_typeerror"
# subject = "tomllib.load"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_misc.py"
# status = "filled"
# ///
"""tomllib.load: str_file_typeerror (errors)."""
import tomllib
import io

_raised = False
try:
    tomllib.load(io.StringIO('a = 1'))
except TypeError:
    _raised = True
assert _raised, "str_file_typeerror: expected TypeError"
print("str_file_typeerror OK")

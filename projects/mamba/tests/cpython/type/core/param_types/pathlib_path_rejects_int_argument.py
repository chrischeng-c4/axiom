# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "pathlib_path_rejects_int_argument"
# subject = "pathlib.Path"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""pathlib.Path: pathlib_path_rejects_int_argument (errors)."""
import pathlib

try:
    result = pathlib.Path(1)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])

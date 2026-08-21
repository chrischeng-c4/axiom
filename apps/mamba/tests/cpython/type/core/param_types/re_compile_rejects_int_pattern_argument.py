# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "re_compile_rejects_int_pattern_argument"
# subject = "re.compile"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""re.compile: re_compile_rejects_int_pattern_argument (errors)."""
import re

try:
    result = re.compile(123)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "hashlib_sha256_rejects_str_argument"
# subject = "hashlib.sha256"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""hashlib.sha256: hashlib_sha256_rejects_str_argument (errors)."""
import hashlib

try:
    result = hashlib.sha256("abc")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "hmac_new_rejects_str_key_argument"
# subject = "hmac.new"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""hmac.new: hmac_new_rejects_str_key_argument (errors)."""
import hmac

try:
    result = hmac.new("key", b"msg")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])

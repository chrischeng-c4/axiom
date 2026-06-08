# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "options_is_int_bitmask"
# subject = "ssl.SSLContext.options"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.options: ctx.options is an int bitmask that supports in-place bitwise-OR with OP_NO_SSLv2 / OP_ALL and stays an int"""
import ssl

_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
_ctx.check_hostname = False
_ctx.verify_mode = ssl.CERT_NONE
assert isinstance(_ctx.options, int), f"options type = {type(_ctx.options)!r}"
_ctx.options |= ssl.OP_NO_SSLv2 if hasattr(ssl, "OP_NO_SSLv2") else ssl.OP_ALL
assert isinstance(_ctx.options, int), f"options type = {type(_ctx.options)!r}"

print("options_is_int_bitmask OK")

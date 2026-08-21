# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "protocol_attr_is_passed_member"
# subject = "ssl.SSLContext.protocol"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.protocol: ctx.protocol is the exact enum member passed to the constructor (identity-preserving)"""
import ssl

_proto = ssl.PROTOCOL_TLS_CLIENT
assert ssl.SSLContext(_proto).protocol is _proto, "ctx.protocol is the member"

print("protocol_attr_is_passed_member OK")

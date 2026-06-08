# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "default_protocol_is_tls"
# subject = "ssl.SSLContext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext: a bare SSLContext() (deprecation warning suppressed) defaults to protocol PROTOCOL_TLS"""
import ssl

import warnings

with warnings.catch_warnings():
    warnings.simplefilter("ignore")
    _ctx = ssl.SSLContext()
assert _ctx.protocol == ssl.PROTOCOL_TLS, "default protocol is PROTOCOL_TLS"

print("default_protocol_is_tls OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "client_server_default_verify_modes"
# subject = "ssl.SSLContext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext: a PROTOCOL_TLS_CLIENT context verifies by default (check_hostname True, verify_mode CERT_REQUIRED) while a PROTOCOL_TLS_SERVER context does not (check_hostname False, verify_mode CERT_NONE)"""
import ssl

_cli = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
assert _cli.check_hostname is True, "client check_hostname default True"
assert _cli.verify_mode == ssl.CERT_REQUIRED, "client verify default REQUIRED"
_srv = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
assert _srv.check_hostname is False, "server check_hostname default False"
assert _srv.verify_mode == ssl.CERT_NONE, "server verify default NONE"

print("client_server_default_verify_modes OK")

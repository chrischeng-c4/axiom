# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "num_tickets_server_mutable"
# subject = "ssl.SSLContext.num_tickets"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.num_tickets: a server context's num_tickets defaults to 2 and is settable to 1, while a client context's num_tickets is fixed at 2"""
import ssl

_srv = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
assert _srv.num_tickets == 2, "server num_tickets default 2"
_srv.num_tickets = 1
assert _srv.num_tickets == 1, "num_tickets settable"
_clt = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
assert _clt.num_tickets == 2, "client num_tickets is 2"

print("num_tickets_server_mutable OK")

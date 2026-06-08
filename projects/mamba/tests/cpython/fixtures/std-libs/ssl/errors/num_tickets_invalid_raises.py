# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "num_tickets_invalid_raises"
# subject = "ssl.SSLContext.num_tickets"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.num_tickets: server num_tickets rejects a negative int with ValueError and None with TypeError, while a client context's num_tickets is read-only and raises ValueError on assignment"""
import ssl

_srv = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
try:
    _srv.num_tickets = -1
    raise AssertionError("negative num_tickets should raise")
except ValueError:
    pass
try:
    _srv.num_tickets = None
    raise AssertionError("None num_tickets should raise")
except TypeError:
    pass

_clt = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
try:
    _clt.num_tickets = 1
    raise AssertionError("client num_tickets should be read-only")
except ValueError:
    pass

print("num_tickets_invalid_raises OK")

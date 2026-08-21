# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "HTTPSServer__init__server_address_as__AfInetAddress_wrong"
# subject = "http.server.HTTPSServer.__init__(server_address: _AfInetAddress)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.HTTPSServer.__init__(server_address: _AfInetAddress); call it with the wrong type.

typeshed contract: server_address is _AfInetAddress. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.server import HTTPSServer
try:
    HTTPSServer(_W(), None)  # server_address: _AfInetAddress <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

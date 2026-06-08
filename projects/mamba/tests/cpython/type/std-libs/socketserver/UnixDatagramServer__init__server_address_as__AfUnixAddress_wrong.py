# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "UnixDatagramServer__init__server_address_as__AfUnixAddress_wrong"
# subject = "socketserver.UnixDatagramServer.__init__(server_address: _AfUnixAddress)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.UnixDatagramServer.__init__(server_address: _AfUnixAddress); call it with the wrong type.

typeshed contract: server_address is _AfUnixAddress. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socketserver import UnixDatagramServer
try:
    UnixDatagramServer(_W(), None)  # server_address: _AfUnixAddress <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

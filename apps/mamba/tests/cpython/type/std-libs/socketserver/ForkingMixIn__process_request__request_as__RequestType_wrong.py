# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "ForkingMixIn__process_request__request_as__RequestType_wrong"
# subject = "socketserver.ForkingMixIn.process_request(request: _RequestType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.ForkingMixIn.process_request(request: _RequestType); call it with the wrong type.

typeshed contract: request is _RequestType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socketserver import ForkingMixIn
obj = object.__new__(ForkingMixIn)
try:
    obj.process_request(_W(), None)  # request: _RequestType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

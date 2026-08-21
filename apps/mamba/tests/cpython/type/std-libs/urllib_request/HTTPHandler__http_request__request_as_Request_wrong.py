# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPHandler__http_request__request_as_Request_wrong"
# subject = "urllib.request.HTTPHandler.http_request(request: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPHandler.http_request(request: Request); call it with the wrong type.

typeshed contract: request is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPHandler
obj = object.__new__(HTTPHandler)
try:
    obj.http_request(_W())  # request: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractHTTPHandler__do_open__http_class_as__HTTPConnectionProtocol_wrong"
# subject = "urllib.request.AbstractHTTPHandler.do_open(http_class: _HTTPConnectionProtocol)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractHTTPHandler.do_open(http_class: _HTTPConnectionProtocol); call it with the wrong type.

typeshed contract: http_class is _HTTPConnectionProtocol. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import AbstractHTTPHandler
obj = object.__new__(AbstractHTTPHandler)
try:
    obj.do_open(_W(), None)  # http_class: _HTTPConnectionProtocol <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

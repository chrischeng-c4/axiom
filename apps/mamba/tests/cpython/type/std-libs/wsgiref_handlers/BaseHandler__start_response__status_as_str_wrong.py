# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_handlers"
# dimension = "type"
# case = "BaseHandler__start_response__status_as_str_wrong"
# subject = "wsgiref.handlers.BaseHandler.start_response(status: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.handlers.BaseHandler.start_response(status: str); call it with the wrong type.

typeshed contract: status is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from wsgiref.handlers import BaseHandler
obj = object.__new__(BaseHandler)
try:
    obj.start_response(12345, None)  # status: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

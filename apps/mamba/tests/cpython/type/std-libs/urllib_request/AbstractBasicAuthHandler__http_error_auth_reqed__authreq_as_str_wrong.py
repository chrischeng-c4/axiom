# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractBasicAuthHandler__http_error_auth_reqed__authreq_as_str_wrong"
# subject = "urllib.request.AbstractBasicAuthHandler.http_error_auth_reqed(authreq: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractBasicAuthHandler.http_error_auth_reqed(authreq: str); call it with the wrong type.

typeshed contract: authreq is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import AbstractBasicAuthHandler
obj = object.__new__(AbstractBasicAuthHandler)
try:
    obj.http_error_auth_reqed(12345, "", None, None)  # authreq: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

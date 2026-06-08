# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractDigestAuthHandler__http_error_auth_reqed__auth_header_as_str_wrong"
# subject = "urllib.request.AbstractDigestAuthHandler.http_error_auth_reqed(auth_header: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractDigestAuthHandler.http_error_auth_reqed(auth_header: str); call it with the wrong type.

typeshed contract: auth_header is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import AbstractDigestAuthHandler
obj = object.__new__(AbstractDigestAuthHandler)
try:
    obj.http_error_auth_reqed(12345, "", None, None)  # auth_header: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

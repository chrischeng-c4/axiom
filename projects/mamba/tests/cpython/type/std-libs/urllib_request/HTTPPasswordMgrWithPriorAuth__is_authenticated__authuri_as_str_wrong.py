# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPPasswordMgrWithPriorAuth__is_authenticated__authuri_as_str_wrong"
# subject = "urllib.request.HTTPPasswordMgrWithPriorAuth.is_authenticated(authuri: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPPasswordMgrWithPriorAuth.is_authenticated(authuri: str); call it with the wrong type.

typeshed contract: authuri is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import HTTPPasswordMgrWithPriorAuth
obj = object.__new__(HTTPPasswordMgrWithPriorAuth)
try:
    obj.is_authenticated(12345)  # authuri: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

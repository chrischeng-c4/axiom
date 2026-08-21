# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPPasswordMgr__find_user_password__realm_as_str_wrong"
# subject = "urllib.request.HTTPPasswordMgr.find_user_password(realm: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPPasswordMgr.find_user_password(realm: str); call it with the wrong type.

typeshed contract: realm is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import HTTPPasswordMgr
obj = object.__new__(HTTPPasswordMgr)
try:
    obj.find_user_password(12345, "")  # realm: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

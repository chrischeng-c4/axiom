# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPPasswordMgr__is_suburi__base_as_str_wrong"
# subject = "urllib.request.HTTPPasswordMgr.is_suburi(base: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPPasswordMgr.is_suburi(base: str); call it with the wrong type.

typeshed contract: base is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import HTTPPasswordMgr
obj = object.__new__(HTTPPasswordMgr)
try:
    obj.is_suburi(12345, "")  # base: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

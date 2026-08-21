# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "CacheFTPHandler__setMaxConns__m_as_int_wrong"
# subject = "urllib.request.CacheFTPHandler.setMaxConns(m: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.CacheFTPHandler.setMaxConns(m: int); call it with the wrong type.

typeshed contract: m is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import CacheFTPHandler
obj = object.__new__(CacheFTPHandler)
try:
    obj.setMaxConns("not_an_int")  # m: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

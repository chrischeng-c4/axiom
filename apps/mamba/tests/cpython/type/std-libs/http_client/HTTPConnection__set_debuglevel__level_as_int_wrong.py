# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "type"
# case = "HTTPConnection__set_debuglevel__level_as_int_wrong"
# subject = "http.client.HTTPConnection.set_debuglevel(level: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.client.HTTPConnection.set_debuglevel(level: int); call it with the wrong type.

typeshed contract: level is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.client import HTTPConnection
obj = object.__new__(HTTPConnection)
try:
    obj.set_debuglevel("not_an_int")  # level: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

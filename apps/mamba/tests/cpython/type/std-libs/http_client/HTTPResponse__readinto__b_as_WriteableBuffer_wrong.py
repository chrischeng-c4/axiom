# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "type"
# case = "HTTPResponse__readinto__b_as_WriteableBuffer_wrong"
# subject = "http.client.HTTPResponse.readinto(b: WriteableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.client.HTTPResponse.readinto(b: WriteableBuffer); call it with the wrong type.

typeshed contract: b is WriteableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.client import HTTPResponse
obj = object.__new__(HTTPResponse)
try:
    obj.readinto(_W())  # b: WriteableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

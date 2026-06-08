# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "type"
# case = "HTTPError__headers__headers_as_Message_wrong"
# subject = "urllib.error.HTTPError.headers(headers: Message)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed headers"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/error.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed headers
# mamba-strict-type: TypeError
"""Type wall: urllib.error.HTTPError.headers(headers: Message); call it with the wrong type.

typeshed contract: headers is Message. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.error import HTTPError
obj = object.__new__(HTTPError)
try:
    obj.headers(_W())  # headers: Message <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

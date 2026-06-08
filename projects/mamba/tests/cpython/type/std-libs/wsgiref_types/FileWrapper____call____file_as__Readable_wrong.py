# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_types"
# dimension = "type"
# case = "FileWrapper____call____file_as__Readable_wrong"
# subject = "wsgiref.types.FileWrapper.__call__(file: _Readable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/types.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.types.FileWrapper.__call__(file: _Readable); call it with the wrong type.

typeshed contract: file is _Readable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.types import FileWrapper
obj = object.__new__(FileWrapper)
try:
    obj.__call__(_W())  # file: _Readable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

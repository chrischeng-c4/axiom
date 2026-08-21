# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_validate"
# dimension = "type"
# case = "WriteWrapper____call____s_as_bytes_wrong"
# subject = "wsgiref.validate.WriteWrapper.__call__(s: bytes)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed s"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/validate.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed s
# mamba-strict-type: TypeError
"""Type wall: wsgiref.validate.WriteWrapper.__call__(s: bytes); call it with the wrong type.

typeshed contract: s is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from wsgiref.validate import WriteWrapper
obj = object.__new__(WriteWrapper)
try:
    obj.__call__(12345)  # s: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "type"
# case = "HMAC__update__msg_as_ReadableBuffer_wrong"
# subject = "hmac.HMAC.update(msg: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/hmac.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: hmac.HMAC.update(msg: ReadableBuffer); call it with the wrong type.

typeshed contract: msg is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from hmac import HMAC
obj = object.__new__(HMAC)
try:
    obj.update(_W())  # msg: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "openssl_sha384__string_as_ReadableBuffer_wrong"
# subject = "_hashlib.openssl_sha384(string: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.openssl_sha384(string: ReadableBuffer); call it with the wrong type.

typeshed contract: string is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import openssl_sha384
try:
    openssl_sha384(_W())  # string: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "urlsafe_b64encode__s_as_ReadableBuffer_wrong"
# subject = "base64.urlsafe_b64encode(s: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.urlsafe_b64encode(s: ReadableBuffer); call it with the wrong type.

typeshed contract: s is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import urlsafe_b64encode
try:
    urlsafe_b64encode(_W())  # s: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

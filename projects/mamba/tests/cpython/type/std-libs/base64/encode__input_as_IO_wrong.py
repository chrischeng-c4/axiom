# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "encode__input_as_IO_wrong"
# subject = "base64.encode(input: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.encode(input: IO); call it with the wrong type.

typeshed contract: input is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import encode
try:
    encode(_W(), None)  # input: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

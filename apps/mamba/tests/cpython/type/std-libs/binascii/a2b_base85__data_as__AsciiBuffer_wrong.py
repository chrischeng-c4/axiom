# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "a2b_base85__data_as__AsciiBuffer_wrong"
# subject = "binascii.a2b_base85(data: _AsciiBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.a2b_base85(data: _AsciiBuffer); call it with the wrong type.

typeshed contract: data is _AsciiBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import a2b_base85
try:
    a2b_base85(_W())  # data: _AsciiBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

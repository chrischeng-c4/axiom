# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "a2b_hex__hexstr_as__AsciiBuffer_wrong"
# subject = "binascii.a2b_hex(hexstr: _AsciiBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.a2b_hex(hexstr: _AsciiBuffer); call it with the wrong type.

typeshed contract: hexstr is _AsciiBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import a2b_hex
try:
    a2b_hex(_W())  # hexstr: _AsciiBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

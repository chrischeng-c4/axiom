# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "type"
# case = "CodecInfo____new____encode_as__Encoder_wrong"
# subject = "codecs.CodecInfo.__new__(encode: _Encoder)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/codecs.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: codecs.CodecInfo.__new__(encode: _Encoder); call it with the wrong type.

typeshed contract: encode is _Encoder. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from codecs import CodecInfo
obj = object.__new__(CodecInfo)
try:
    obj.__new__(_W(), None)  # encode: _Encoder <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

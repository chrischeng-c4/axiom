# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteStreamWriter__write__strobj_as_str_wrong"
# subject = "_multibytecodec.MultibyteStreamWriter.write(strobj: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteStreamWriter.write(strobj: str); call it with the wrong type.

typeshed contract: strobj is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _multibytecodec import MultibyteStreamWriter
obj = object.__new__(MultibyteStreamWriter)
try:
    obj.write(12345)  # strobj: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

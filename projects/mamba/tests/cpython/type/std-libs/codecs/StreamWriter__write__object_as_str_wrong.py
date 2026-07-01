# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "type"
# case = "StreamWriter__write__object_as_str_wrong"
# subject = "codecs.StreamWriter.write(object: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/codecs.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: codecs.StreamWriter.write(object: str); call it with the wrong type.

typeshed contract: object is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from codecs import StreamWriter
obj = object.__new__(StreamWriter)
try:
    obj.write(12345)  # object: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

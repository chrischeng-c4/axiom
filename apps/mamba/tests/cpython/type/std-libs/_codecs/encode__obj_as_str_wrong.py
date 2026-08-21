# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_codecs"
# dimension = "type"
# case = "encode__obj_as_str_wrong"
# subject = "_codecs.encode(obj: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_codecs.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _codecs.encode(obj: str); call it with the wrong type.

typeshed contract: obj is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _codecs import encode
try:
    encode(12345, None)  # obj: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json_encoder"
# dimension = "type"
# case = "encode_basestring__s_as_str_wrong"
# subject = "json.encoder.encode_basestring(s: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json/encoder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: json.encoder.encode_basestring(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from json.encoder import encode_basestring
try:
    encode_basestring(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

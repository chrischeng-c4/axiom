# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json_decoder"
# dimension = "type"
# case = "JSONDecodeError__init__msg_as_str_wrong"
# subject = "json.decoder.JSONDecodeError.__init__(msg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json/decoder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: json.decoder.JSONDecodeError.__init__(msg: str); call it with the wrong type.

typeshed contract: msg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from json.decoder import JSONDecodeError
try:
    JSONDecodeError(12345, "", 0)  # msg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

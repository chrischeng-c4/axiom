# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "parse_mime_parameters__value_as_str_wrong"
# subject = "email._header_value_parser.parse_mime_parameters(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.parse_mime_parameters(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import parse_mime_parameters
try:
    parse_mime_parameters(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "type"
# case = "LineTooLong__init__line_type_as_str_wrong"
# subject = "http.client.LineTooLong.__init__(line_type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.client.LineTooLong.__init__(line_type: str); call it with the wrong type.

typeshed contract: line_type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.client import LineTooLong
try:
    LineTooLong(12345)  # line_type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

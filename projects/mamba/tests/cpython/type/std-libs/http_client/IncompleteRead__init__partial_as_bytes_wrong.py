# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "type"
# case = "IncompleteRead__init__partial_as_bytes_wrong"
# subject = "http.client.IncompleteRead.__init__(partial: bytes)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.client.IncompleteRead.__init__(partial: bytes); call it with the wrong type.

typeshed contract: partial is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.client import IncompleteRead
try:
    IncompleteRead(12345)  # partial: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

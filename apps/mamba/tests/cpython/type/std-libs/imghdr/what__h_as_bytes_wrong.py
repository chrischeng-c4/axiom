# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imghdr"
# dimension = "type"
# case = "what__h_as_bytes_wrong"
# subject = "imghdr.what(h: bytes)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/imghdr.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: imghdr.what(h: bytes); call it with the wrong type.

typeshed contract: h is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from imghdr import what
try:
    what(None, 12345)  # h: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "nntplib"
# dimension = "type"
# case = "decode_header__header_str_as_str_wrong"
# subject = "nntplib.decode_header(header_str: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/nntplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: nntplib.decode_header(header_str: str); call it with the wrong type.

typeshed contract: header_str is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from nntplib import decode_header
try:
    decode_header(12345)  # header_str: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

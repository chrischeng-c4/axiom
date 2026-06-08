# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_header"
# dimension = "type"
# case = "Header__encode__splitchars_as_str_wrong"
# subject = "email.header.Header.encode(splitchars: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/header.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.header.Header.encode(splitchars: str); call it with the wrong type.

typeshed contract: splitchars is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.header import Header
obj = object.__new__(Header)
try:
    obj.encode(12345)  # splitchars: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_charset"
# dimension = "type"
# case = "Charset__header_encode_lines__string_as_str_wrong"
# subject = "email.charset.Charset.header_encode_lines(string: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/charset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.charset.Charset.header_encode_lines(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.charset import Charset
obj = object.__new__(Charset)
try:
    obj.header_encode_lines(12345, None)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

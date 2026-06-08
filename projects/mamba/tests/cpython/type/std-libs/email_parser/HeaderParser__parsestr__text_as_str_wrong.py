# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_parser"
# dimension = "type"
# case = "HeaderParser__parsestr__text_as_str_wrong"
# subject = "email.parser.HeaderParser.parsestr(text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.parser.HeaderParser.parsestr(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.parser import HeaderParser
obj = object.__new__(HeaderParser)
try:
    obj.parsestr(12345)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

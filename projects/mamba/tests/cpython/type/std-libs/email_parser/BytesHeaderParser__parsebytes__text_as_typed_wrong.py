# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_parser"
# dimension = "type"
# case = "BytesHeaderParser__parsebytes__text_as_typed_wrong"
# subject = "email.parser.BytesHeaderParser.parsebytes(text: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.parser.BytesHeaderParser.parsebytes(text: typed); call it with the wrong type.

typeshed contract: text is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.parser import BytesHeaderParser
obj = object.__new__(BytesHeaderParser)
try:
    obj.parsebytes(_W())  # text: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_parser"
# dimension = "type"
# case = "BytesParser__parse__fp_as__WrappedBuffer_wrong"
# subject = "email.parser.BytesParser.parse(fp: _WrappedBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.parser.BytesParser.parse(fp: _WrappedBuffer); call it with the wrong type.

typeshed contract: fp is _WrappedBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.parser import BytesParser
obj = object.__new__(BytesParser)
try:
    obj.parse(_W())  # fp: _WrappedBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

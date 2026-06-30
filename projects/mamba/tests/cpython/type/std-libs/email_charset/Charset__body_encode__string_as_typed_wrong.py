# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_charset"
# dimension = "type"
# case = "Charset__body_encode__string_as_typed_wrong"
# subject = "email.charset.Charset.body_encode(string: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/charset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.charset.Charset.body_encode(string: typed); call it with the wrong type.

typeshed contract: string is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.charset import Charset
obj = object.__new__(Charset)
try:
    obj.body_encode(_W())  # string: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

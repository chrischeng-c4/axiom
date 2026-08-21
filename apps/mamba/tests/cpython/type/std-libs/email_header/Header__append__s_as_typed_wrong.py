# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_header"
# dimension = "type"
# case = "Header__append__s_as_typed_wrong"
# subject = "email.header.Header.append(s: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/header.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.header.Header.append(s: typed); call it with the wrong type.

typeshed contract: s is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.header import Header
obj = object.__new__(Header)
try:
    obj.append(_W())  # s: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_header"
# dimension = "type"
# case = "decode_header__header_as_typed_wrong"
# subject = "email.header.decode_header(header: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/header.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.header.decode_header(header: typed); call it with the wrong type.

typeshed contract: header is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.header import decode_header
try:
    decode_header(_W())  # header: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_base64mime"
# dimension = "type"
# case = "header_encode__header_bytes_as_typed_wrong"
# subject = "email.base64mime.header_encode(header_bytes: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/base64mime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.base64mime.header_encode(header_bytes: typed); call it with the wrong type.

typeshed contract: header_bytes is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.base64mime import header_encode
try:
    header_encode(_W())  # header_bytes: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_encoders"
# dimension = "type"
# case = "encode_quopri__msg_as_Message_wrong"
# subject = "email.encoders.encode_quopri(msg: Message)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/encoders.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.encoders.encode_quopri(msg: Message); call it with the wrong type.

typeshed contract: msg is Message. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.encoders import encode_quopri
try:
    encode_quopri(_W())  # msg: Message <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

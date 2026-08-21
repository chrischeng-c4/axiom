# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_message"
# dimension = "type"
# case = "Message__set_payload__payload_as_typed_wrong"
# subject = "email.message.Message.set_payload(payload: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/message.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.message.Message.set_payload(payload: typed); call it with the wrong type.

typeshed contract: payload is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.message import Message
obj = object.__new__(Message)
try:
    obj.set_payload(_W())  # payload: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

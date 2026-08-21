# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_message"
# dimension = "type"
# case = "Message__as_string__unixfrom_as_bool_wrong"
# subject = "email.message.Message.as_string(unixfrom: bool)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/message.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.message.Message.as_string(unixfrom: bool); call it with the wrong type.

typeshed contract: unixfrom is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.message import Message
obj = object.__new__(Message)
try:
    obj.as_string("not_a_bool")  # unixfrom: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

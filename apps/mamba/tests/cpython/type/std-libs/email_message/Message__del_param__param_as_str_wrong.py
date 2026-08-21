# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_message"
# dimension = "type"
# case = "Message__del_param__param_as_str_wrong"
# subject = "email.message.Message.del_param(param: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/message.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.message.Message.del_param(param: str); call it with the wrong type.

typeshed contract: param is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.message import Message
obj = object.__new__(Message)
try:
    obj.del_param(12345)  # param: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

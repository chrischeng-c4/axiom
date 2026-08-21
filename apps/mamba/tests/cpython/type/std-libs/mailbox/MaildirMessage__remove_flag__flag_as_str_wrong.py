# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MaildirMessage__remove_flag__flag_as_str_wrong"
# subject = "mailbox.MaildirMessage.remove_flag(flag: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MaildirMessage.remove_flag(flag: str); call it with the wrong type.

typeshed contract: flag is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import MaildirMessage
obj = object.__new__(MaildirMessage)
try:
    obj.remove_flag(12345)  # flag: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MaildirMessage__set_subdir__subdir_as_Literal_wrong"
# subject = "mailbox.MaildirMessage.set_subdir(subdir: Literal)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed subdir"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed subdir
# mamba-strict-type: TypeError
"""Type wall: mailbox.MaildirMessage.set_subdir(subdir: Literal); call it with the wrong type.

typeshed contract: subdir is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import MaildirMessage
obj = object.__new__(MaildirMessage)
try:
    obj.set_subdir(_W())  # subdir: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

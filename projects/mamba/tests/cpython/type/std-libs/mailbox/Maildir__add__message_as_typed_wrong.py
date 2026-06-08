# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir__add__message_as_typed_wrong"
# subject = "mailbox.Maildir.add(message: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.add(message: typed); call it with the wrong type.

typeshed contract: message is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.add(_W())  # message: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

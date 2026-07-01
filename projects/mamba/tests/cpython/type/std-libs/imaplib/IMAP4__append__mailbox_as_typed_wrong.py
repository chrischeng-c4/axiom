# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imaplib"
# dimension = "type"
# case = "IMAP4__append__mailbox_as_typed_wrong"
# subject = "imaplib.IMAP4.append(mailbox: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/imaplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: imaplib.IMAP4.append(mailbox: typed); call it with the wrong type.

typeshed contract: mailbox is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from imaplib import IMAP4
obj = object.__new__(IMAP4)
try:
    obj.append(_W(), None, None, None)  # mailbox: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

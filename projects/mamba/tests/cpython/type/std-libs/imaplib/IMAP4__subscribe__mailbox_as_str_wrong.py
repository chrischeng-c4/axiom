# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imaplib"
# dimension = "type"
# case = "IMAP4__subscribe__mailbox_as_str_wrong"
# subject = "imaplib.IMAP4.subscribe(mailbox: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/imaplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: imaplib.IMAP4.subscribe(mailbox: str); call it with the wrong type.

typeshed contract: mailbox is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from imaplib import IMAP4
obj = object.__new__(IMAP4)
try:
    obj.subscribe(12345)  # mailbox: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

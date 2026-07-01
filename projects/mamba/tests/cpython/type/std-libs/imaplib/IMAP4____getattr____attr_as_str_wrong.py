# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imaplib"
# dimension = "type"
# case = "IMAP4____getattr____attr_as_str_wrong"
# subject = "imaplib.IMAP4.__getattr__(attr: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/imaplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: imaplib.IMAP4.__getattr__(attr: str); call it with the wrong type.

typeshed contract: attr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from imaplib import IMAP4
obj = object.__new__(IMAP4)
try:
    obj.__getattr__(12345)  # attr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

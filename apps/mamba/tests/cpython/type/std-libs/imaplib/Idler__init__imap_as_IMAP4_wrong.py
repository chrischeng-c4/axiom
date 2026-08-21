# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imaplib"
# dimension = "type"
# case = "Idler__init__imap_as_IMAP4_wrong"
# subject = "imaplib.Idler.__init__(imap: IMAP4)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/imaplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: imaplib.Idler.__init__(imap: IMAP4); call it with the wrong type.

typeshed contract: imap is IMAP4. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from imaplib import Idler
try:
    Idler(_W())  # imap: IMAP4 <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

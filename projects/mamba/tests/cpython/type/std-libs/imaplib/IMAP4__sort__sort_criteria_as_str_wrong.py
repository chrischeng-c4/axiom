# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imaplib"
# dimension = "type"
# case = "IMAP4__sort__sort_criteria_as_str_wrong"
# subject = "imaplib.IMAP4.sort(sort_criteria: str)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed sort_criteria"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/imaplib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed sort_criteria
# mamba-strict-type: TypeError
"""Type wall: imaplib.IMAP4.sort(sort_criteria: str); call it with the wrong type.

typeshed contract: sort_criteria is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from imaplib import IMAP4
obj = object.__new__(IMAP4)
try:
    obj.sort(12345, "")  # sort_criteria: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

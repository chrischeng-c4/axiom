# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "nntplib"
# dimension = "type"
# case = "NNTP__newnews__group_as_str_wrong"
# subject = "nntplib.NNTP.newnews(group: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/nntplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: nntplib.NNTP.newnews(group: str); call it with the wrong type.

typeshed contract: group is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from nntplib import NNTP
obj = object.__new__(NNTP)
try:
    obj.newnews(12345, None)  # group: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

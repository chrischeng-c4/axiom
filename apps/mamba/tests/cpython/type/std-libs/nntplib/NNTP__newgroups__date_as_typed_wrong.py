# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "nntplib"
# dimension = "type"
# case = "NNTP__newgroups__date_as_typed_wrong"
# subject = "nntplib.NNTP.newgroups(date: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/nntplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: nntplib.NNTP.newgroups(date: typed); call it with the wrong type.

typeshed contract: date is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from nntplib import NNTP
obj = object.__new__(NNTP)
try:
    obj.newgroups(_W())  # date: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "isascii__c_as_typed_wrong"
# subject = "curses.ascii.isascii(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.isascii(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import isascii
try:
    isascii(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

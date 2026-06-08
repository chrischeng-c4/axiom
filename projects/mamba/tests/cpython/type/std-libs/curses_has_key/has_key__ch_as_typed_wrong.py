# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_has_key"
# dimension = "type"
# case = "has_key__ch_as_typed_wrong"
# subject = "curses.has_key.has_key(ch: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/has_key.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.has_key.has_key(ch: typed); call it with the wrong type.

typeshed contract: ch is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.has_key import has_key
try:
    has_key(_W())  # ch: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

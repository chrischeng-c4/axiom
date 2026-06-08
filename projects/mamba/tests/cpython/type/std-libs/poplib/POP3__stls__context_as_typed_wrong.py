# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poplib"
# dimension = "type"
# case = "POP3__stls__context_as_typed_wrong"
# subject = "poplib.POP3.stls(context: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/poplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: poplib.POP3.stls(context: typed); call it with the wrong type.

typeshed contract: context is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from poplib import POP3
obj = object.__new__(POP3)
try:
    obj.stls(_W())  # context: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncore"
# dimension = "type"
# case = "dispatcher__bind__addr_as_typed_wrong"
# subject = "asyncore.dispatcher.bind(addr: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed addr"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncore.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed addr
# mamba-strict-type: TypeError
"""Type wall: asyncore.dispatcher.bind(addr: typed); call it with the wrong type.

typeshed contract: addr is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncore import dispatcher
obj = object.__new__(dispatcher)
try:
    obj.bind(_W())  # addr: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

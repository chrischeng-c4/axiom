# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "type"
# case = "serialize_iterator__throw__typ_as_type_wrong"
# subject = "threading.serialize_iterator.throw(typ: type)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed typ"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/threading.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed typ
# mamba-strict-type: TypeError
"""Type wall: threading.serialize_iterator.throw(typ: type); call it with the wrong type.

typeshed contract: typ is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from threading import serialize_iterator
obj = object.__new__(serialize_iterator)
try:
    obj.throw(_W())  # typ: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

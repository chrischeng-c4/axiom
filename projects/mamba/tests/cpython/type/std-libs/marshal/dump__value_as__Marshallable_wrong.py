# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "type"
# case = "dump__value_as__Marshallable_wrong"
# subject = "marshal.dump(value: _Marshallable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/marshal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: marshal.dump(value: _Marshallable); call it with the wrong type.

typeshed contract: value is _Marshallable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from marshal import dump
try:
    dump(_W(), None)  # value: _Marshallable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

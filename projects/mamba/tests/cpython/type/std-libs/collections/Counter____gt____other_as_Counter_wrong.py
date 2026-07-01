# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "type"
# case = "Counter____gt____other_as_Counter_wrong"
# subject = "collections.Counter.__gt__(other: Counter)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/collections.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: collections.Counter.__gt__(other: Counter); call it with the wrong type.

typeshed contract: other is Counter. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from collections import Counter
obj = object.__new__(Counter)
try:
    obj.__gt__(_W())  # other: Counter <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

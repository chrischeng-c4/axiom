# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "type"
# case = "Random__choice__seq_as_SupportsLenAndGetItem_wrong"
# subject = "random.Random.choice(seq: SupportsLenAndGetItem)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/random.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: random.Random.choice(seq: SupportsLenAndGetItem); call it with the wrong type.

typeshed contract: seq is SupportsLenAndGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from random import Random
obj = object.__new__(Random)
try:
    obj.choice(_W())  # seq: SupportsLenAndGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

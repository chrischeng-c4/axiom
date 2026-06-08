# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "type"
# case = "NormalDist____rsub____x2_as_typed_wrong"
# subject = "statistics.NormalDist.__rsub__(x2: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/statistics.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: statistics.NormalDist.__rsub__(x2: typed); call it with the wrong type.

typeshed contract: x2 is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from statistics import NormalDist
obj = object.__new__(NormalDist)
try:
    obj.__rsub__(_W())  # x2: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
